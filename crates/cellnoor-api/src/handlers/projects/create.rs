use axum::{Json, extract::State};
use cellnoor_types::project::{NewProject, ProjectDetailed, ProjectField};

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

async fn insert_project(
    tx: &db::Transaction<'_>,
    new: &NewProject,
) -> Result<ProjectDetailed, DbError> {
    // `created_by` is omitted: the database fills it from `app_user_id()`
    let id = tx.insert_returning_id(new).await?;

    insert_project_accesses(tx, id, &new.members).await?;

    select_project_by_id(tx, id).await
}

impl Insert for NewProject {
    type Field = ProjectField;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        use ProjectField::*;

        let Self {
            name,
            started_at,
            ended_at,
            members: _,
        } = self;

        vec![(Name, name), (StartedAt, started_at), (EndedAt, ended_at)]
    }
}

#[cfg(test)]
pub mod test {

    use cellnoor_types::project::{NewProject, ProjectDetailed};
    use jiff::Timestamp;
    use uuid::Uuid;

    use crate::{
        db::{self, DbError},
        handlers::{
            people::create::test::insert_test_person_and_institution,
            projects::create::insert_project,
        },
        state::dev_util::{ToNonemptyString, db_client_as_admin},
    };

    // This one returns a `Result` because a different test needs that
    pub async fn insert_test_project<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewProject, ProjectDetailed), DbError>
    where
        F: FnMut(&mut NewProject),
    {
        // The fixture's person is valid, so only the database can refuse it,
        // and no test here reads that error
        let (_, person) = insert_test_person_and_institution(tx, |_| ())
            .await
            .expect("failed to insert the test person");
        let person_id = person.record.id;

        let mut new = NewProject {
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            started_at: Timestamp::now(),
            ended_at: Timestamp::MAX,
            members: vec![person_id],
        };

        modify(&mut new);

        let inserted = insert_project(tx, &new).await?;

        Ok((new, inserted))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        insert_test_project(&tx, |_| ()).await.unwrap();
    }
}
