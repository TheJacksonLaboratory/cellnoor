use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::project::{NewProject, ProjectDetailed};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self},
    error::{Error, ErrorInner},
    handlers::{
        path::IdParam,
        projects::{access::add_person::insert_project_accesses, show::select_project_by_id},
    },
    state::AppState,
};

pub async fn update_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    Json(project): Json<NewProject>,
) -> Result<Json<ProjectDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_project_by_id(&tx, id, &project).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn update_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    NewProject { record, people }: &NewProject,
) -> Result<ProjectDetailed, ErrorInner> {
    db::update(tx, "project", id, record).await?;

    insert_project_accesses(tx, id, people).await?;

    select_project_by_id(tx, id).await
}

#[cfg(test)]
mod tests {
    use crate::{
        handlers::projects::{create::test::insert_test_project, update::update_project_by_id},
        state::test_util::{ToNonemptyString, db_client_as_admin},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn update() {
        let mut client = db_client_as_admin().await;
        let tx = client.begin().await.unwrap();

        let (pre_update, inserted) = insert_test_project(&tx, |_| ()).await.unwrap();
        let mut update = pre_update;
        update.record.name = "updated".to_nonempty_string();
        update.people = vec![];

        update_project_by_id(&tx, *inserted.record.project.id, &update)
            .await
            .unwrap();
    }
}
