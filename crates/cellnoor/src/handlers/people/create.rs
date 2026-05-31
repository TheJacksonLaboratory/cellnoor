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
    db::{self, AsFieldValuePairs, SqlBuilder},
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
        permissions_to_revoke,
    }: &NewPerson,
) -> Result<Person, ErrorInner> {
    validate_email(record.email.as_ref().map(NonemptyString::as_ref))?;

    let id = db::insert_into(tx, "person", record).await?;

    let (_, person) = tokio::try_join!(
        provision_db_user(tx, id, permissions_to_grant, permissions_to_revoke),
        select_person_by_id(tx, id)
    )?;

    Ok(person)
}

impl AsFieldValuePairs<PersonField, 5> for NewPersonRecord {
    fn as_field_value_pairs(&self) -> db::FieldValuePairs<'_, PersonField, 5> {
        use PersonField::*;

        let Self {
            id: _,
            name,
            email,
            institution_id,
            is_staff,
            orcid,
        } = self;

        [
            (Name, name),
            (InstitutionId, institution_id),
            (Email, email),
            (IsStaff, is_staff),
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
    permissions_to_grant: &PermissionsToGrant,
    permissions_to_revoke: &PermissionsToRevoke,
) -> Result<(), ErrorInner> {
    tx.acquire_user_permisssions_lock().await?;

    create_db_user(tx, person_id).await?;
    let can_create_person = modify_create_person_permissions(
        tx,
        person_id,
        permissions_to_grant,
        permissions_to_revoke,
    )
    .await?;
    grant_permissions_to_db_user(tx, person_id, permissions_to_grant, can_create_person).await?;
    revoke_permissions_from_db_user(tx, person_id, permissions_to_revoke).await?;

    Ok(())
}

async fn create_db_user(tx: &db::Transaction<'_>, user_id: Uuid) -> Result<(), ErrorInner> {
    static PROVISION_USER: SqlBuilder =
        SqlBuilder::new("select create_person_user_if_not_exists($1)");

    let user_id = user_id.to_string();
    let sql = PROVISION_USER.finish_with_params(vec![&user_id]);

    tx.execute(&sql).await?;

    Ok(())
}

async fn modify_create_person_permissions(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    grant_permissions: &PermissionsToGrant,
    revoke_permissions: &PermissionsToRevoke,
) -> Result<bool, ErrorInner> {
    let filter_person_creation_perms = |p: &&ResourcePermission| matches!(*p, ResourcePermission::Person(a) if a.contains(&Action::Create));

    let grant_create_person = grant_permissions
        .iter()
        .filter(filter_person_creation_perms)
        .map(Clone::clone)
        .collect::<Vec<_>>()
        .into();

    let revoke_create_person = revoke_permissions
        .iter()
        .filter(filter_person_creation_perms)
        .map(Clone::clone)
        .collect::<Vec<_>>()
        .into();

    // We pass false here because we haven't run the complete set of operations that
    // may revoke the user's 'create person' privilege. This will be remedied
    // because we run another set of grants later
    grant_permissions_to_db_user(tx, user_id, &grant_create_person, false).await?;
    revoke_permissions_from_db_user(tx, user_id, &revoke_create_person).await?;

    let can_create_person = user_can_create_person(tx, user_id).await?;
    if can_create_person {
        tx.execute_raw_sql(&format!(r#"alter user "{user_id}" with createrole"#), &[])
            .await?;
    } else {
        tx.execute_raw_sql(&format!(r#"alter user "{user_id}" with nocreaterole"#), &[])
            .await?;
    };

    Ok(can_create_person)
}

async fn grant_permissions_to_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &PermissionsToGrant,
    can_grant_to_others: bool,
) -> Result<(), ErrorInner> {
    let grant_stmts: Vec<_> = permissions
        .iter()
        .map(|p| construct_grant_statement(user_id, p, can_grant_to_others))
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
    let revoke_stmts: Vec<_> = permissions
        .iter()
        .map(|p| construct_revoke_statement(user_id, p))
        .collect();

    let revoke_ops = revoke_stmts.iter().map(|s| tx.execute_raw_sql(s, &[]));
    futures::future::try_join_all(revoke_ops).await?;

    Ok(())
}

async fn user_can_create_person(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
) -> Result<bool, ErrorInner> {
    static SELECT_HAS_TABLE_PRIVILEGE: SqlBuilder =
        SqlBuilder::new("select has_table_privilege($1, 'person', 'insert')");

    Ok(tx
        .query_one_into(&SELECT_HAS_TABLE_PRIVILEGE.finish_with_params(vec![&user_id.to_string()]))
        .await?)
}

#[derive(Clone, Copy, strum::Display, PartialEq)]
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

fn construct_grant_statement(
    user_id: Uuid,
    resource_permissions: &ResourcePermission,
    can_grant_to_others: bool,
) -> String {
    construct_grant_or_revoke_statement(
        GrantOrRevoke::Grant,
        user_id,
        resource_permissions,
        can_grant_to_others,
    )
}

fn construct_revoke_statement(user_id: Uuid, resource_permissions: &ResourcePermission) -> String {
    construct_grant_or_revoke_statement(GrantOrRevoke::Revoke, user_id, resource_permissions, false)
}

fn construct_grant_or_revoke_statement(
    grant_or_revoke: GrantOrRevoke,
    user_id: Uuid,
    resource_permissions: &ResourcePermission,
    with_grant: bool,
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

    let suffix = if with_grant && grant_or_revoke == GrantOrRevoke::Grant {
        "with grant option"
    } else {
        ""
    };

    format!(
        r#"{grant_or_revoke} {actions} on {resource_names} {} "{user_id}" {suffix}"#,
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
                is_staff: false,
                orcid: None,
            },
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
        std::assert_matches!(error, ErrorInner::PermissionDenied { .. });
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

        // Log in as the created person and create another person with that permission
        let mut client = db_client_as_user(*person.record.id).await;
        let tx = client.begin().await.unwrap();

        insert_test_person_and_institution(&tx, grant_create_person)
            .await
            .unwrap();

        // Also try to grant a permission that this user doesn't actually have
        let error = insert_test_person_and_institution(&tx, |p| {
            p.permissions_to_grant = vec![ResourcePermission::Project(vec![Action::Create])].into()
        })
        .await
        .unwrap_err();

        std::assert_matches!(error, ErrorInner::PermissionDenied { .. });
    }
}
