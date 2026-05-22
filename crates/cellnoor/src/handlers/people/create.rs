use std::sync::LazyLock;

use axum::{Json, extract::State};
use cellnoor_types::person::{
    Action, NewPerson, NewPersonRecord, PermissionsToGrant, PermissionsToRevoke, Person,
    PersonField, ResourcePermission,
};
use nonempty::NonemptyString;
use regex::Regex;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, BaseSqlStmt},
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

pub async fn insert_person(
    tx: &db::Transaction<'_>,
    NewPerson {
        record,
        is_staff,
        permissions_to_grant,
        permissions_to_revoke,
    }: &NewPerson,
) -> Result<Person, ErrorInner> {
    validate_email(record.email.as_ref().map(NonemptyString::as_ref))?;

    let id = db::insert_into(tx, "person", record).await?;

    let (_, person) = tokio::try_join!(
        provision_db_user(
            tx,
            id,
            *is_staff,
            permissions_to_grant,
            permissions_to_revoke
        ),
        select_person_by_id(tx, id)
    )?;

    Ok(person)
}

impl AsFieldValuePairs<PersonField, 4> for NewPersonRecord {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, PersonField, 4> {
        use PersonField::*;

        let Self {
            id: _,
            name,
            email,
            institution_id,
            orcid,
        } = self;

        [
            (Name, name),
            (InstitutionId, institution_id),
            (Email, email),
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

pub(super) async fn provision_db_user(
    tx: &db::Transaction<'_>,
    person_id: Uuid,
    is_staff: bool,
    permissions_to_grant: &PermissionsToGrant,
    permissions_to_revoke: &PermissionsToRevoke,
) -> Result<(), ErrorInner> {
    tx.acquire_user_permisssions_lock().await?;

    create_db_user(tx, person_id, is_staff).await?;
    grant_permissions_to_db_user(tx, person_id, permissions_to_grant).await?;
    revoke_permissions_from_db_user(tx, person_id, permissions_to_revoke).await?;

    Ok(())
}

async fn create_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    is_staff: bool,
) -> Result<(), ErrorInner> {
    let user_id = user_id.to_string();
    let sql = BaseSqlStmt::new("select create_person_user_if_not_exists($1, $2)")
        .finish_with_params(vec![&user_id, &is_staff]);

    tx.execute(&sql).await?;

    Ok(())
}

async fn grant_permissions_to_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &PermissionsToGrant,
) -> Result<(), ErrorInner> {
    let grant_stmts: Vec<_> = permissions
        .iter()
        .map(|p| construct_grant_or_revoke_statement(GrantOrRevoke::Grant, user_id, p))
        .collect();

    let grant_ops = grant_stmts.iter().map(|s| tx.execute_raw_sql(s, &[]));
    futures::future::try_join_all(grant_ops).await?;

    Ok(())
}

async fn revoke_permissions_from_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &PermissionsToRevoke,
) -> Result<(), ErrorInner> {
    let revoke_stmt: Vec<_> = permissions
        .iter()
        .map(|p| construct_grant_or_revoke_statement(GrantOrRevoke::Revoke, user_id, p))
        .collect();

    let revoke_ops = revoke_stmt.iter().map(|s| tx.execute_raw_sql(s, &[]));
    futures::future::try_join_all(revoke_ops).await?;

    Ok(())
}

#[derive(Clone, Copy, strum::Display)]
#[strum(serialize_all = "snake_case")]
enum GrantOrRevoke {
    Grant,
    Revoke,
}
impl GrantOrRevoke {
    fn preposition(self) -> &'static str {
        match self {
            Self::Grant => "to",
            Self::Revoke => "from",
        }
    }
}

fn construct_grant_or_revoke_statement(
    grant_or_revoke: GrantOrRevoke,
    user_id: Uuid,
    resource_permissions: &ResourcePermission,
) -> String {
    let resource_names = permission_as_tableset(resource_permissions);
    let actions = match resource_permissions {
        ResourcePermission::Institution(a)
        | ResourcePermission::Person(a)
        | ResourcePermission::Project(a)
        | ResourcePermission::Specimen(a)
        | ResourcePermission::AssayConstantData(a)
        | ResourcePermission::ChromiumExperimentalData(a)
        | ResourcePermission::ChromiumDataset(a) => a,
    };

    let actions: Vec<_> = actions.iter().map(Action::as_str).collect();
    let actions = actions.join(", ");

    format!(
        r#"{grant_or_revoke} {actions} on {resource_names} {} "{user_id}""#,
        grant_or_revoke.preposition()
    )
}

fn permission_as_tableset(permission: &ResourcePermission) -> &'static str {
    match permission {
        ResourcePermission::Institution(_) => "institution",
        ResourcePermission::Person(_) => "person",
        ResourcePermission::Project(_) => "project",
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

#[cfg(test)]
pub mod test {

    use cellnoor_types::{
        id::NoId,
        person::{Action, NewPerson, NewPersonRecord, Person, ResourcePermission},
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
                orcid: None,
            },
            is_staff: false,
            permissions_to_grant: vec![].into(),
            permissions_to_revoke: vec![].into(),
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
            p.permissions_to_grant = vec![
                ResourcePermission::Institution(vec![Action::Create]),
                ResourcePermission::Person(vec![Action::Create]),
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

        // TODO: once we have `insert_test_chromium_dataset`, use that to ensure the
        // user can insert everything required in the chain from an institution to a
        // Chromium dataset

        // Check that they cannot insert a project
        let error = insert_test_project(&tx, |_| ()).await.unwrap_err();
        assert_eq!(error, ErrorInner::PermissionDenied);
    }

    // We only test this once in the earliest place in the "chain"
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
}
