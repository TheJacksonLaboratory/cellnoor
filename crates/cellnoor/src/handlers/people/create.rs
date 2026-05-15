use std::sync::LazyLock;

use axum::{Json, extract::State};
use cellnoor_types::person::{
    NewPerson, NewPersonRecord, PermissionsToGrant, PermissionsToRevoke, Person, PersonField,
    ResourcePermission,
};
use nonempty::NonemptyString;
use regex::Regex;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, ToRecord},
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

    // But they can be grouped and done concurrently with the select
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

impl ToRecord<PersonField, 4> for NewPersonRecord {
    fn to_record(&self) -> db::Record<PersonField, 4> {
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
    tx.execute(
        "select create_person_user_if_not_exists($1, $2)",
        &[&user_id.to_string(), &is_staff],
    )
    .await?;

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

    let grant_ops = grant_stmts.iter().map(|s| tx.execute(s, &[]));
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

    let revoke_ops = revoke_stmt.iter().map(|s| tx.execute(s, &[]));
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
    let resource_name = resource_permissions.as_ref();
    let actions = match resource_permissions {
        ResourcePermission::Institution(a)
        | ResourcePermission::Person(a)
        | ResourcePermission::Project(a)
        | ResourcePermission::Specimen(a)
        | ResourcePermission::ChromiumExperimentalEntities(a)
        | ResourcePermission::ChromiumDataset(a) => a,
    };

    let actions: Vec<_> = actions.iter().map(|a| a.as_ref()).collect();
    let actions = actions.join(", ");

    format!(
        r#"{grant_or_revoke} {actions} on {resource_name} {} "{user_id}""#,
        grant_or_revoke.preposition()
    )
}

#[cfg(test)]
pub mod test {
    use std::convert::identity;

    use cellnoor_types::{
        id::NoId,
        institution::InstitutionQuery,
        person::{
            Action, NewPerson, NewPersonRecord, Person, ResourcePermission, SavedPersonRecord,
        },
        project::ProjectQuery,
    };
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            institutions::{create::test::insert_test_institution, index::select_institutions},
            people::create::insert_person,
            projects::{
                create::test::insert_test_project, index::select_projects,
                show::select_project_by_id,
            },
        },
        state::test_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
    };

    pub async fn insert_test_person_and_institution<F>(
        tx: &db::Transaction<'_>,
        modify: F,
    ) -> (NewPerson, Person)
    where
        F: FnMut(&mut NewPerson),
    {
        let (_, institution) = insert_test_institution(tx, identity).await;

        let mut new = NewPerson {
            record: NewPersonRecord {
                id: NoId {},
                name: "hamood".to_nonempty_string(),
                institution_id: *institution.record.id,
                email: Some(format!("{}@jax.org", Uuid::new_v4()).to_nonempty_string()),
                orcid: None,
            },
            is_staff: false,
            permissions_to_grant: vec![ResourcePermission::Institution(vec![Action::Create])]
                .into(),
            permissions_to_revoke: vec![].into(),
        };

        modify(&mut new);

        let inserted = insert_person(tx, &new).await.unwrap();
        (new, inserted)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_person_and_institution(&tx, identity).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_permissions() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Create a new person and db-user
        let (
            _,
            Person {
                record: SavedPersonRecord { id: user_id, .. },
                ..
            },
        ) = insert_test_person_and_institution(&tx, identity).await;

        let (_, accessible_project) =
            insert_test_project(&tx, |new| new.people = vec![*user_id]).await;

        // And insert one the new user cannot
        let (_, inaccessible_project) = insert_test_project(&tx, identity).await;

        // We have to commit this transaction so the change persists for the next part
        // of the test
        tx.commit().await.unwrap();

        // Log in as the new user
        let mut client = db_client_as_user(*user_id).await;
        let tx = client.begin().await.unwrap();

        // Check that the user can do what they should be able to
        select_institutions(&tx, &InstitutionQuery::default())
            .await
            .unwrap();
        let _ = insert_test_institution(&tx, identity).await;

        let error = insert_test_project(&tx, identity).await.unwrap_err();
        assert_eq!(error, ErrorInner::PermissionDenied);
        tx.commit().await.unwrap();

        let tx = client.begin().await.unwrap();
        // Check that only one of the two projects is accessible
        let projects = select_projects(
            &tx,
            &ProjectQuery {
                detailed: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(projects, vec![accessible_project]);

        // Check that the inaccessible project causes a `ResourceNotFound`
        let error = select_project_by_id(&tx, *inaccessible_project.record().id)
            .await
            .unwrap_err();

        assert_eq!(error, ErrorInner::ResourceNotFound);
    }

    // We only test this once in the earliest place in the "chain"
    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_reference_error() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let new = NewPerson {
            record: NewPersonRecord {
                id: NoId {},
                name: "hamood".to_nonempty_string(),
                institution_id: Uuid::new_v4(),
                email: Some(format!("{}@jax.org", Uuid::new_v4()).to_nonempty_string()),
                orcid: None,
            },
            is_staff: false,
            permissions_to_grant: vec![].into(),
            permissions_to_revoke: vec![].into(),
        };

        let error = insert_person(&tx, &new).await.unwrap_err();

        assert_eq!(
            error,
            ErrorInner::InvalidReference {
                referencing_resource: "person".to_owned(),
                referencing_field: "institution_id".to_owned(),
            },
        );
    }
}
