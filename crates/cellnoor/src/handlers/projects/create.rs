use axum::{Json, extract::State};
use cellnoor_types::project::{NewProject, NewProjectRecord, Project, ProjectField};

use crate::{
    auth::AuthUser,
    db::{self, Record, ToRecord},
    error::{Error, ErrorInner},
    handlers::projects::{access::add_person::insert_project_accesses, show::select_project_by_id},
    state::AppState,
};

pub async fn create_project(
    State(state): State<AppState>,
    user: AuthUser,
    Json(project): Json<NewProject>,
) -> Result<Json<Project>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_project(&tx, &project).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn insert_project(
    tx: &db::Transaction<'_>,
    NewProject { record, people }: &NewProject,
) -> Result<Project, ErrorInner> {
    let id = db::insert_into(tx, "project", record).await?;

    insert_project_accesses(tx, id, people).await?;

    select_project_by_id(tx, id).await
}

impl ToRecord<ProjectField, 3> for NewProjectRecord {
    fn to_record(&self) -> Record<'_, ProjectField, 3> {
        use ProjectField::*;

        let Self {
            id: _,
            name,
            started_at,
            ended_at,
        } = self;

        [(Name, name), (StartedAt, started_at), (EndedAt, ended_at)]
    }
}

#[cfg(test)]
pub mod test {

    use cellnoor_types::{
        id::NoId,
        project::{NewProject, NewProjectRecord, Project},
    };
    use jiff::Timestamp;
    use uuid::Uuid;

    use crate::{
        db,
        error::ErrorInner,
        handlers::{
            people::create::test::insert_test_person_and_institution,
            projects::create::insert_project,
        },
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    // This one returns a `Result` because a different test needs that
    pub async fn insert_test_project<F>(
        tx: &db::Transaction<'_>,
        mut modify: F,
    ) -> Result<(NewProject, Project), ErrorInner>
    where
        F: FnMut(&mut NewProject),
    {
        let (_, person) = insert_test_person_and_institution(tx, |_| ()).await?;
        let person_id = *person.record.id;

        let mut new = NewProject {
            record: NewProjectRecord {
                id: NoId {},
                name: Uuid::new_v4().to_string().to_nonempty_string(),
                started_at: Timestamp::now(),
                ended_at: Timestamp::now(),
            },
            people: vec![person_id],
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
