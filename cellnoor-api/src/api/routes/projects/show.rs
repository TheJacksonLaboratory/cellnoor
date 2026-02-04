use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, project::Project};
use cellnoor_schema::projects;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn show_project(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Project>, db::Error> {
    select_project_by_id(id, &mut db_conn).await.map(Json)
}

async fn select_project_by_id(
    project_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Project, db::Error> {
    Ok(Project::query()
        .filter(projects::id.eq(project_id))
        .first(&mut db_conn)
        .await?)
}
