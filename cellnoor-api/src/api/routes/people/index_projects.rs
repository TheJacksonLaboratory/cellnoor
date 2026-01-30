use axum::{
    Extension, Json,
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
    api::{
        auth::{AuthenticatedUser, RemoveUnauthorizedProjects},
        extract::JsonQuery,
    },
    db::{self, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub async fn index_person_projects(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Extension(user): Extension<AuthenticatedUser>,
    Path(IdParameter { id }): Path<IdParameter>,
    JsonQuery { mut q }: JsonQuery<ProjectQuery>,
) -> Result<Json<Vec<Project>>, db::Error> {
    q.filter.ids.remove_unauthorized_projects(&user);

    select_person_projects(id, q, &mut db_conn).await.map(Json)
}

pub async fn select_person_projects(
    person_id: Uuid,
    ProjectQuery {
        filter,
        limit,
        offset,
        order_by,
    }: ProjectQuery,
    db_conn: &mut DbConnection,
) -> Result<Vec<Project>, db::Error> {
    let mut stmt = Project::query()
        .inner_join(project_people::table)
        .filter(project_people::person_id.eq(person_id))
        .filter(filter.to_boxed_filter())
        .limit(limit)
        .offset(offset)
        .into_boxed();

    for ordering in order_by {
        stmt = stmt.order_by(ordering);
    }

    Ok(stmt.load(db_conn).await?)
}
