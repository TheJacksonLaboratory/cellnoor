use std::sync::LazyLock;

use axum::{Json, extract::State};
use cellnoor_types::person::{
    Account, Action, NewPerson, PermissionsToGrant, Person, PersonField, PersonSimpleFields,
    ResourcePermission,
};
use nonempty::NonemptyString;
use postgres_types::ToSql;
use regex::Regex;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs},
    error::{Error, ErrorInner},
    handlers::people::show::select_person_by_id,
    state::AppState,
};

pub async fn create_person(
    State(state): State<AppState>,
    user: AuthUser,
    Json(person): Json<NewPerson>,
) -> Result<Json<Person>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_person(&tx, &person).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_person(
    tx: &db::Transaction<'_>,
    NewPerson {
        simple,
        account,
        permissions_to_grant,
    }: &NewPerson,
) -> Result<Person, ErrorInner> {
    match account {
        Account::None { email } => validate_email(email.as_ref())?,
        Account::Microsoft {
            microsoft_entra_oid: _,
        } => (),
    };

    let id = db::insert_into(tx, "person", simple).await?;

    // These 3 operations can happen concurrently because they don't depend on one another (inshallah)
    let (person, _, _) = tokio::try_join!(
        select_person_by_id(tx, id),
        insert_account(tx, id, account),
        create_db_user(tx, id, permissions_to_grant)
    )?;

    Ok(person)
}

async fn insert_account(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    account: &Account,
) -> Result<(), ErrorInner> {
    let res = match account {
        Account::Microsoft {
            microsoft_entra_oid,
        } => {
            db::insert_into_no_returning(
                tx,
                "account",
                &NewAccountRecord::new(user_id, account.as_ref(), microsoft_entra_oid),
            )
            .await?
        }
        Account::None { email: _ } => (),
    };

    Ok(res)
}

async fn create_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &PermissionsToGrant,
) -> Result<(), ErrorInner> {
    let permission_sets: Vec<_> = permissions
        .iter()
        .map(permission_to_permission_set)
        .collect();

    tx.execute_raw_sql(
        "select create_person_user_with_permissions($1, $2)",
        &[&user_id, &permission_sets],
    )
    .await?;

    Ok(())
}

#[derive(Debug, ToSql)]
#[postgres(name = "permission_set")]
pub(in super::super) struct PermissionSet {
    pub tableset: &'static str,
    pub actions: String,
}

pub(in super::super) fn permission_to_permission_set(
    permission: &ResourcePermission,
) -> PermissionSet {
    PermissionSet {
        tableset: permission_as_tableset(permission),
        actions: action_list_to_str(permission),
    }
}

fn permission_as_tableset(permission: &ResourcePermission) -> &'static str {
    match permission {
        ResourcePermission::Institution(_) => "institution",
        ResourcePermission::Person(_) => "person",
        ResourcePermission::Account(_) => "account, person_account",
        ResourcePermission::Project(_) => "project, project_access",
        ResourcePermission::Specimen(_) => "specimen",
        ResourcePermission::AssayConstantData(_) => {
            "tenx_assay, index_kit, single_index_set, dual_index_set, library_type_specification, \
             multiplexing_tag"
        }
        ResourcePermission::ChromiumExperimentalData(_) => {
            "suspension, suspension_measurement, suspension_preparer, suspension_pool, \
             suspension_pool_measurement, suspension_pool_preparer, chromium_run, gem_well, \
             chip_loading, cdna, cdna_measurement, cdna_preparer, library, library_measurement, \
             library_preparer"
        }
        ResourcePermission::ChromiumDataset(_) => {
            "chromium_dataset, chromium_dataset_raw_file, chromium_dataset_parsed_file, \
             chromium_dataset_library"
        }
    }
}

fn action_list_to_str(permission: &ResourcePermission) -> String {
    let actions = match permission {
        ResourcePermission::Institution(a)
        | ResourcePermission::Person(a)
        | ResourcePermission::Account(a)
        | ResourcePermission::Project(a)
        | ResourcePermission::Specimen(a)
        | ResourcePermission::AssayConstantData(a)
        | ResourcePermission::ChromiumExperimentalData(a)
        | ResourcePermission::ChromiumDataset(a) => a,
    };

    let actions: Vec<&str> = actions.iter().map(Action::as_ref).collect();
    actions.join(", ")
}

impl AsFieldValuePairs<PersonField, 5> for PersonSimpleFields {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, PersonField, 5> {
        use PersonField::*;

        let Self {
            name,
            institution_id,
            is_staff,
            can_manage_users,
            orcid,
        } = self;

        [
            (Name, name),
            (InstitutionId, institution_id),
            (IsStaff, is_staff),
            (CanManageUsers, can_manage_users),
            (Orcid, orcid),
        ]
    }
}

impl AsFieldValuePairs<PersonField, 6> for NewPerson {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, PersonField, 6> {
        use PersonField::*;

        let Self {
            simple,
            account,
            permissions_to_grant: _,
        } = self;

        let email: &(dyn ToSql + Sync) = match account {
            Account::Microsoft {
                microsoft_entra_oid: _,
            } => &None::<NonemptyString>,
            Account::None { email } => email,
        };

        // Initialize an array with dummy info
        let mut fields: [(PersonField, &(dyn ToSql + Sync)); _] = [(PersonField::Name, &""); 6];

        fields[..5].copy_from_slice(&simple.as_field_value_pairs());

        fields[5] = (Email, email);

        fields
    }
}

struct NewAccountRecord<'a> {
    person_id: Uuid,
    auth_provider: &'a str,
    auth_provider_user_id: String,
}

impl<'a> NewAccountRecord<'a> {
    fn new(
        person_id: Uuid,
        auth_provider: &'a str,
        auth_provider_user_id: impl std::fmt::Display,
    ) -> Self {
        Self {
            person_id,
            auth_provider,
            auth_provider_user_id: auth_provider_user_id.to_string(),
        }
    }
}

impl AsFieldValuePairs<&'static str, 3> for NewAccountRecord<'_> {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, &'static str, 3> {
        let Self {
            person_id,
            auth_provider,
            auth_provider_user_id,
        } = self;

        [
            ("person_id", person_id),
            ("auth_provider", auth_provider),
            ("auth_provider_user_id", auth_provider_user_id),
        ]
    }
}

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

pub(super) fn validate_email(email: &str) -> Result<(), ErrorInner> {
    if !EMAIL_REGEX.is_match(email) {
        return Err(ErrorInner::DataConstraint {
            resource: Some("person".to_owned()),
            field: Some("email".to_owned()),
            message: "invalid email".to_owned(),
            detail: None,
        });
    }

    Ok(())
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::person::{
        Account, Action, NewPerson, PermissionsToGrant, Person, PersonSimpleFields,
        ResourcePermission,
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            institutions::create::test::insert_test_institution, people::create::insert_person,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
    };

    pub async fn insert_test_person_and_institution<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewPerson, Person), ErrorInner>
    where
        F: FnMut(&mut NewPerson),
    {
        let (_, institution) = insert_test_institution(tx, |_| ()).await?;

        let mut new = NewPerson {
            simple: PersonSimpleFields {
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                institution_id: *institution.record.id,
                is_staff: false,
                can_manage_users: false,
                orcid: None,
            },
            account: Account::Microsoft {
                microsoft_entra_oid: Uuid::new_v4(),
            },
            permissions_to_grant: PermissionsToGrant::default(),
        };

        modify(&mut new);

        let inserted = insert_person(tx, &new).await?;
        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_person_and_institution(&tx, |_| ())
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_permissions() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Create a new person with permissions to create every entity in the chain from
        // an institution to a Chromium dataset
        let (_, person) = insert_test_person_and_institution(&tx, |p| {
            let NewPerson {
                simple,
                permissions_to_grant,
                account: _,
            } = p;

            simple.is_staff = false;
            *permissions_to_grant = vec![
                ResourcePermission::Institution(vec![Action::Create]),
                ResourcePermission::Person(vec![Action::Create, Action::Update, Action::Delete]),
                ResourcePermission::Project(vec![Action::Create]),
                ResourcePermission::Specimen(vec![Action::Create]),
                ResourcePermission::AssayConstantData(vec![Action::Create]),
                ResourcePermission::ChromiumExperimentalData(vec![Action::Create]),
                ResourcePermission::ChromiumExperimentalData(vec![Action::Create]),
            ]
            .into();
        })
        .await
        .unwrap();

        // Commit the transaction so the change persists for the next part of the test
        tx.commit().await.unwrap();

        // Log in as the new user
        let mut client = db_client_as_user(person.record.id).await;
        let tx = client.begin().await.unwrap();

        // Ensure they can insert an institution (since they were granted permission to
        // do so)
        insert_test_institution(&tx, |_| ()).await.unwrap();
    }

    // This test just ensures that the correct error is returned when there's an
    // invalid foreign key
    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_reference_error() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let error = insert_test_person_and_institution(&tx, |p| {
            p.simple.institution_id = Uuid::new_v4();
        })
        .await
        .unwrap_err();

        assert_eq!(
            error,
            ErrorInner::InvalidReference {
                referencing_resource: "person".to_owned(),
                referencing_field: "institution_id".to_owned(),
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn user_can_grant_to_others() {
        let create = vec![Action::Create];

        let grant_create_person = |p: &mut NewPerson| {
            p.simple.can_manage_users = true;
            p.permissions_to_grant = vec![
                ResourcePermission::Institution(create.clone()),
                ResourcePermission::Person(create.clone()),
                ResourcePermission::Account(create.clone()),
            ]
            .into()
        };

        // First, create a person that can create other people
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (_, person) = insert_test_person_and_institution(&tx, grant_create_person)
            .await
            .unwrap();

        tx.commit().await.unwrap();

        // Log in as the created person
        let mut client = db_client_as_user(person.record.id).await;
        let tx = client.begin().await.unwrap();

        // The newly created person should be able to create another person with the
        // requested permissions
        let (_, person) = insert_test_person_and_institution(&tx, |p| {
            p.permissions_to_grant =
                vec![ResourcePermission::Institution(vec![Action::Create])].into()
        })
        .await
        .unwrap();

        tx.commit().await.unwrap();

        // Finally, log in as the most newly created person and ensure they have
        // permission to do what they've been granted
        let mut client = db_client_as_user(person.record.id).await;
        let tx = client.begin().await.unwrap();

        insert_test_institution(&tx, |_| ()).await.unwrap();
    }
}
