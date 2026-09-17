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
    db::{self, DbError},
    handlers::{IdParam, projects::index_detailed::select_projects_detailed},
    state::AppState,
};

pub async fn show_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<ProjectDetailed>, DbError> {
    state
        .in_transaction(user, async |tx| select_project_by_id(tx, id).await)
        .await
}

// Visibility required for tests
pub(super) async fn select_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<ProjectDetailed, DbError> {
    tx.select_one(
        ProjectPredicate::Id(UuidOperator::Eq(id)),
        select_projects_detailed,
    )
    .await
}
