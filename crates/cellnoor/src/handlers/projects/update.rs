use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::project::{NewProject, ProjectDetailed, ProjectField};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::{Error, ErrorInner},
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
    Json(project): Json<NewProject>,
) -> Result<Json<ProjectDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = update_project_by_id(&tx, id, &project).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

async fn update_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
    updated_project: &NewProject,
) -> Result<ProjectDetailed, ErrorInner> {
    db::update(tx, "project", id, &ProjectUpdate(updated_project)).await?;

    insert_project_accesses(tx, id, &updated_project.members).await?;

    select_project_by_id(tx, id).await
}

// It's tempting to just reuse the `impl` for NewProjectWithCreator (see
// ../create.rs) but we don't want to change the project creator once it's
// created
struct ProjectUpdate<'a>(&'a NewProject);

impl AsFieldValuePairs<ProjectField, 3> for ProjectUpdate<'_> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, ProjectField, 3> {
        use ProjectField::*;

        let Self(NewProject {
            name,
            started_at,
            ended_at,
            members: _,
        }) = self;

        [(Name, name), (StartedAt, started_at), (EndedAt, ended_at)]
    }
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
