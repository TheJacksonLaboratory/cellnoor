use std::sync::LazyLock;

use axum::{Json, extract::State};
use cellnoor_types::person::{
    Action, NewPerson, NewPersonRecord, PermissionsToGrant, Person,
    PersonField, ResourcePermission,
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
        record,
        permissions_to_grant,
    }: &NewPerson,
) -> Result<Person, ErrorInner> {
    validate_email(record.email.as_ref().map(NonemptyString::as_ref))?;

    let id = db::insert_into(tx, "person", record).await?;

    let person = select_person_by_id(&tx, id).await?;

    create_db_user(tx, *person.record.id, permissions_to_grant).await?;

    Ok(person)
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
        | ResourcePermission::Project(a)
        | ResourcePermission::Specimen(a)
        | ResourcePermission::AssayConstantData(a)
        | ResourcePermission::ChromiumExperimentalData(a)
        | ResourcePermission::ChromiumDataset(a) => a,
    };

    let actions: Vec<_> = actions.iter().map(Action::as_str).collect();
    actions.join(", ")
}

impl AsFieldValuePairs<PersonField, 6> for NewPersonRecord {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, PersonField, 6> {
        use PersonField::*;

        let Self {
            id: _,
            name,
            email,
            institution_id,
            can_read_all_projects,
            can_admin_users,
            orcid,
        } = self;

        [
            (Name, name),
            (InstitutionId, institution_id),
            (Email, email),
            (CanReadAllProjects, can_read_all_projects),
            (CanAdminUsers, can_admin_users),
            (Orcid, orcid),
        ]
    }
}

// https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9.!#$%&'*+\/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap()
});

pub(super) fn validate_email(email: Option<&str>) -> Result<(), ErrorInner> {
    let error = ErrorInner::DataConstraint {
        resource: Some("person".to_owned()),
        field: Some("email".to_owned()),
        message: "invalid email".to_owned(),
        detail: None,
    };

    let Some(email) = email else {
        return Err(error);
    };

    if !EMAIL_REGEX.is_match(email) {
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::{
        id::NoId,
        person::{
            Action, NewPerson, NewPersonRecord, PermissionsToGrant, Person, ResourcePermission,
        },
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            institutions::create::test::insert_test_institution, people::create::insert_person,
            projects::create::test::insert_test_project,
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
            record: NewPersonRecord {
                id: NoId {},
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                institution_id: *institution.record.id,
                email: Some(format!("{}@jax.org", Uuid::new_v4()).to_nonempty_string()),
                can_read_all_projects: false,
                can_admin_users: false,
                orcid: None,
            },
            permissions_to_grant: vec![].into(),
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
            p.record.can_read_all_projects = false;
            p.permissions_to_grant = vec![
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
        let mut client = db_client_as_user(*person.record.id).await;
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

        let error =
            insert_test_person_and_institution(&tx, |p| p.record.institution_id = Uuid::new_v4())
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
            p.record.can_admin_users = true;
            p.permissions_to_grant = vec![
                ResourcePermission::Institution(create.clone()),
                ResourcePermission::Person(create.clone()),
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
        let mut client = db_client_as_user(*person.record.id).await;
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
        let mut client = db_client_as_user(*person.record.id).await;
        let tx = client.begin().await.unwrap();

        insert_test_institution(&tx, |_| ()).await.unwrap();
    }
}
