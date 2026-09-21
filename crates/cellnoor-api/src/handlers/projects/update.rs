use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::project::{NewProject, ProjectDetailed};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, DbError},
    handlers::{
        IdParam,
        projects::{access::add_people::insert_project_accesses, show::select_project_by_id},
    },
    state::AppState,
};

pub async fn update_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
    crate::extract::JsonExtractor(project): crate::extract::JsonExtractor<NewProject>,
) -> Result<Json<ProjectDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| {
            update_project_by_id(tx, id, &project).await
        })
        .await
}

async fn update_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    updated_project: &NewProject,
) -> Result<ProjectDetailed, DbError> {
    tx.update(id, updated_project).await?;

    insert_project_accesses(tx, id, &updated_project.members).await?;

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
        update.name = "updated".to_nonempty_string();
        update.members = vec![];

        update_project_by_id(&tx, inserted.record.project.id, &update)
            .await
            .unwrap();
    }
}
