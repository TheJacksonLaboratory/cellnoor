use axum::{Json, extract::State};
use cellnoor_types::project::{NewProject, ProjectDetailed, ProjectField, ProjectSimpleFields};

use crate::{
    auth::AuthUser,
    db::{self, DbError, FieldValues, Insert},
    handlers::projects::{access::add_people::insert_project_accesses, show::select_project_by_id},
    state::AppState,
};

pub async fn create_project(
    State(state): State<AppState>,
    user: AuthUser,
    crate::extract::JsonExtractor(project): crate::extract::JsonExtractor<NewProject>,
) -> Result<Json<ProjectDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| insert_project(tx, &project).await)
        .await
}

pub(in super::super) async fn insert_project(
    tx: &db::Transaction<'_>,
    new: &NewProject,
) -> Result<ProjectDetailed, DbError> {
    // `created_by` is omitted: the database fills it from `app_user_id()`
    let id = tx.insert_returning_id(&new.simple).await?;

    insert_project_accesses(tx, id, &new.members).await?;

    select_project_by_id(tx, id).await
}

impl Insert for ProjectSimpleFields {
    type Field = ProjectField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use ProjectField::*;

        let Self {
            name,
            started_at,
            ended_at,
        } = self;

        vec![(Name, name), (StartedAt, started_at), (EndedAt, ended_at)]
    }
}

// This one returns a `Result` because a different test needs that
#[cfg(any(test, feature = "dev"))]
pub async fn insert_test_project<F>(
    tx: &db::Transaction<'_>,
    mut modify: F,
) -> Result<(NewProject, ProjectDetailed), DbError>
where
    F: FnMut(&mut NewProject),
{
    use jiff::Timestamp;

    use crate::handlers::people::create::insert_test_person_and_institution;

    let (_, person) = insert_test_person_and_institution(tx, |_| ())
        .await
        .unwrap();
    let person_id = person.record.id;

    let mut new = NewProject {
        simple: ProjectSimpleFields {
            name: crate::db::dummy_data::random_name_for("project"),
            started_at: Timestamp::now(),
            ended_at: Timestamp::MAX,
        },
        members: vec![person_id],
    };

    modify(&mut new);

    let inserted = insert_project(tx, &new).await?;

    Ok((new, inserted))
}

#[cfg(test)]
pub mod test {

    use pretty_assertions::assert_eq;

    use super::insert_test_project;
    use crate::{
        db::DbError,
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_project(&tx, |_| ()).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn name_must_be_path_component() {
        let mut client = db_client_as_admin().await;

        for name in [".", "..", "parent/child"] {
            let tx = client.begin().await.unwrap();

            let error = insert_test_project(&tx, |p| p.simple.name = name.to_nonempty_string())
                .await
                .unwrap_err();

            assert_eq!(
                error,
                DbError::DataConstraint {
                    resource: Some("project".to_owned()),
                    field: None,
                    message: "new row for relation \"project\" violates check constraint \
                              \"name_is_path_component\""
                        .to_owned(),
                    detail: None,
                }
            );
        }
    }
}
