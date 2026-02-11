use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    project::{Project, ProjectQuery},
};
use cellnoor_schema::project_people;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::extract::AuthJsonQuery,
    db::{self, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub async fn index_person_projects(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    AuthJsonQuery { q }: AuthJsonQuery<ProjectQuery>,
) -> Result<Json<Vec<Project>>, db::Error> {
    select_person_projects(id, q, &db_conn).await.map(Json)
}

pub async fn select_person_projects(
    person_id: Uuid,
    ProjectQuery {
        filter,
        limit,
        offset,
        order_by,
    }: ProjectQuery,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<Project>, db::Error> {
    let mut stmt = Project::query()
        .inner_join(project_people::table)
        .filter(project_people::person_id.eq(person_id))
        .filter(filter.to_boxed_filter())
        .limit(limit)
        .offset(offset)
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(&mut db_conn).await?)
}
