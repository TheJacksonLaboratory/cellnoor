use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    operator::UuidOperator,
    project::{ProjectDetailed, ProjectPredicate},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, select_one},
    error::{Error, ErrorInner},
    handlers::{IdParam, projects::index_detailed::select_projects_detailed},
    state::AppState,
};

pub async fn show_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<ProjectDetailed>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_project_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

// Visibility required for tests
pub(super) async fn select_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<ProjectDetailed, ErrorInner> {
    select_one(
        tx,
        ProjectPredicate::Id(UuidOperator::Eq(id)),
        select_projects_detailed,
    )
    .await
}
