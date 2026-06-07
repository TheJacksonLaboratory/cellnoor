use axum::{Json, extract::State};
use cellnoor_types::{
    api_key::{PersonId, ServiceId},
    project::{NewProject, ProjectDetailed, ProjectField},
};

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
    handlers::projects::{access::add_people::insert_project_accesses, show::select_project_by_id},
    state::AppState,
};

pub async fn create_project(
    State(state): State<AppState>,
    user: AuthUser,
    Json(project): Json<NewProject>,
) -> Result<Json<ProjectDetailed>, Error> {
    let mut client = state.db_client(user).await?;

    let tx = client.begin().await?;

    let response = insert_project(&tx, &project).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn insert_project(
    tx: &db::Transaction<'_>,
    new: &NewProject,
) -> Result<ProjectDetailed, ErrorInner> {
    let user = tx.user();
    let id = db::insert_into(
        tx,
        "project",
        &NewProjectWithCreator {
            record: new,
            created_by_person: user.person_id(),
            created_by_service: user.service_id(),
        },
    )
    .await?;

    insert_project_accesses(tx, id, &new.members).await?;

    select_project_by_id(tx, id).await
}

struct NewProjectWithCreator<'a> {
    record: &'a NewProject,
    created_by_person: Option<PersonId>,
    created_by_service: Option<ServiceId>,
}

impl AsFieldValuePairs<ProjectField, 5> for NewProjectWithCreator<'_> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, ProjectField, 5> {
        use ProjectField::*;

        let Self {
            record:
                NewProject {
                    name,
                    started_at,
                    ended_at,
                    members: _,
                },
            created_by_person,
            created_by_service,
        } = self;

        [
            (Name, name),
            (StartedAt, started_at),
            (EndedAt, ended_at),
            (CreatedByPerson, created_by_person),
            (CreatedByService, created_by_service),
        ]
    }
}

#[cfg(test)]
pub mod test {

    use cellnoor_types::project::{NewProject, ProjectDetailed};
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
    ) -> Result<(NewProject, ProjectDetailed), ErrorInner>
    where
        F: FnMut(&mut NewProject),
    {
        let (_, person) = insert_test_person_and_institution(tx, |_| ()).await?;
        let person_id = *person.record.id;

        let mut new = NewProject {
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            started_at: Timestamp::now(),
            ended_at: Timestamp::now(),
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
