use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    UuidOperator,
    project::{Project, ProjectPredicate},
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db::{self, util::select_one},
    error::{Error, ErrorInner},
    handlers::{path::IdParam, projects::index::select_projects},
    state::AppState,
};

pub async fn show_project(
    State(state): State<AppState>,
    user: AuthUser,
    Path(IdParam { id }): Path<IdParam>,
) -> Result<Json<Project>, Error> {
    let mut client = state.db_client(user).await?;
    let tx = client.begin().await?;

    let response = select_project_by_id(&tx, id).await.map(Json)?;

    tx.commit().await?;

    Ok(response)
}

pub async fn select_project_by_id(
    tx: &db::Transaction<'_>,
    id: Uuid,
) -> Result<Project, ErrorInner> {
    select_one(
        tx,
        ProjectPredicate::Id(UuidOperator::Eq(id)),
        select_projects,
    )
    .await
}
