use axum::{Json, extract::State};
use cellnoor_types::project::{NewProject, Project};

use crate::{
    auth::AuthUser,
    db::{
        self,
    },
    error::Error,
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

    let response = insert_project(&tx, &project).await.map(Json);

    tx.commit().await?;

    response
}

pub async fn insert_project(
    tx: &db::Transaction<'_>,
    NewProject {
        name,
        started_at,
        ended_at,
        people,
    }: &NewProject,
) -> Result<Project, crate::error::Error> {
    let id = tx
        .query_one_into(
            "insert into project (name, started_at, ended_at) values ($1, $2, $3) returning id",
            &[name, started_at, ended_at],
        )
        .await?;

    insert_project_accesses(&tx, id, &people).await?;

    select_project_by_id(tx, id).await
}

#[cfg(test)]
pub mod test {
    use cellnoor_types::project::{NewProject, Project, ProjectQuery, ProjectRecordDetailed};
    use jiff::Timestamp;
    use uuid::Uuid;

    use crate::{
        handlers::projects::{create::insert_project, index::select_projects},
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    pub fn new_project() -> NewProject {
        NewProject {
            name: Uuid::new_v4().to_string().to_nonempty_string(),
            started_at: Timestamp::now(),
            ended_at: Timestamp::now(),
            people: vec![],
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn insert_and_select() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let new = new_project();
        let inserted = insert_project(&tx, &new).await.unwrap();

        let mut query = ProjectQuery::default();
        let records = select_projects(&tx, &query).await.unwrap();

        let (
            Project::Detailed {
                record:
                    ProjectRecordDetailed {
                        project: inserted_record,
                        ..
                    },
                ..
            },
            Project::Compact {
                record: selected_record,
                ..
            },
        ) = (&inserted, &records[0])
        else {
            unreachable!(
                "a default select statement should return a `Project::Compact`, while an insert \
                 should return a `Project::Detailed`"
            );
        };

        assert_eq!(inserted_record, selected_record);

        query.detailed = true;
        let mut records = select_projects(&tx, &query).await.unwrap();

        assert_eq!(inserted, records.swap_remove(0));
    }
}
