use axum::{Json, extract::State};
use cellnoor_models::project::{NewProject, Project};
use cellnoor_schema::projects;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_project(
    _: State<AppState>,
    db_conn: DbConnection,
    Json(project): Json<NewProject>,
) -> Result<Json<Project>, db::Error> {
    insert_project(project, &db_conn).await.map(Json)
}

pub async fn insert_project(
    project: NewProject,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Project, db::Error> {
    Ok(diesel::insert_into(projects::table)
        .values(project)
        .returning(Project::as_returning())
        .get_result(&mut db_conn)
        .await?)
}
