use std::sync::LazyLock;

use axum::{Json, extract::State};
use cellnoor_types::person::{NewPerson, NewPersonRecord, Person, ResourcePermission};
use nonempty::NonemptyString;
use regex::Regex;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{
        self,
        util::{FieldValuePairs, ToFieldListPlaceholdersParams},
    },
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
        record:
            NewPersonRecord {
                id: _,
                name,
                email,
                institution_id,
                orcid,
            },

        is_staff,
        grant_permissions: permissions_to_grant,
        revoke_permissions: permissions_to_revoke,
    }: &NewPerson,
) -> Result<Person, ErrorInner> {
    validate_email(email.as_ref().map(NonemptyString::as_ref))?;

    let fields: FieldValuePairs<_> = [
        ("name", name),
        ("institution_id", institution_id),
        ("email", email),
        ("orcid", orcid),
    ];
    let (field_list, placeholders, params) = fields.to_field_list_and_placeholders_and_params();

    // Simple queries can be written inline
    let person_id = tx
        .query_one_into(
            &format!("insert into person {field_list} values {placeholders} returning id"),
            &params,
        )
        .await?;

    let db_user_operations = async || {
        // In the unlikely event that this route is called in a crazily-concurrent
        // fashion, we acquire a transaction-level lock to prevent the error "tuple
        // concurrently updated"

        tx.acquire_user_permisssions_lock().await?;

        create_db_user(tx, person_id, *is_staff).await?;
        grant_permissions_to_db_user(tx, person_id, permissions_to_grant).await?;
        revoke_permissions_from_db_user(tx, person_id, permissions_to_revoke).await?;

        Ok(())
    };

    // But they can be grouped and done concurrently with the select
    let (_, person) = tokio::try_join!(db_user_operations(), select_person_by_id(tx, person_id))?;

    Ok(person)
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

pub(super) async fn create_db_user(
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

pub(super) async fn grant_permissions_to_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &[ResourcePermission],
) -> Result<(), ErrorInner> {
    let grant_stmts: Vec<_> = permissions
        .iter()
        .map(|p| construct_grant_or_revoke_statement(GrantOrRevoke::Grant, user_id, p))
        .collect();

    let grant_ops = grant_stmts.iter().map(|s| tx.execute(s, &[]));
    futures::future::try_join_all(grant_ops).await?;

    Ok(())
}

pub(super) async fn revoke_permissions_from_db_user(
    tx: &db::Transaction<'_>,
    user_id: Uuid,
    permissions: &[ResourcePermission],
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
        error::ErrorInner,
        handlers::{
            institutions::{
                create::{insert_institution, test::new_institution},
                index::select_institutions,
            },
            people::create::insert_person,
            projects::{
                create::{insert_project, test::new_project},
                index::select_projects,
                show::select_project_by_id,
            },
        },
        state::test_util::{ToNonemptyString, db_client_as_admin, db_client_as_user},
    };

    pub fn new_person() -> NewPerson {
        NewPerson {
            record: NewPersonRecord {
                id: NoId {},
                name: "hamood".to_nonempty_string(),
                institution_id: Uuid::nil(),
                email: Some(format!("{}@jax.org", Uuid::new_v4()).to_nonempty_string()),
                orcid: None,
            },
            is_staff: false,
            grant_permissions: vec![ResourcePermission::Institution(vec![Action::Create])],
            revoke_permissions: vec![],
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_with_permissions() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        // Create a new person/db-user
        let new_record = new_person();
        let Person {
            record: SavedPersonRecord { id: user_id, .. },
            ..
        } = insert_person(&tx, &new_record).await.unwrap();

        let mut p = new_project();
        p.people.push(*user_id);
        let accessible_project = insert_project(&tx, &p).await.unwrap();

        // And insert one the new user cannot
        let p = new_project();
        let inaccessible_project = insert_project(&tx, &p).await.unwrap();

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
        insert_institution(&tx, &new_institution()).await.unwrap();

        // They should not be able to insert a project
        let p = new_project();
        let error = insert_project(&tx, &p).await.unwrap_err();
        assert_eq!(error, ErrorInner::PermissionDenied);
        // Commit the transaction because the error causes it to abort
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

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_reference_error() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let mut new_record = new_person();
        new_record.record.institution_id = Uuid::new_v4();

        let error = insert_person(&tx, &new_record).await.unwrap_err();

        assert_eq!(
            error,
            ErrorInner::InvalidReference {
                referencing_resource: "person".to_owned(),
                referencing_field: "institution_id".to_owned(),
            },
        );
    }
}
