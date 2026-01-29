use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    person::{PersonOrderBy, PersonQuery, PersonSummary},
};
use cellnoor_schema::{people, project_people};
use diesel::{pg::Pg, prelude::*};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::extract::JsonQuery,
    db::{self, BoxedFilter, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub async fn index_project_people(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    JsonQuery { q: query }: JsonQuery<PersonQuery>,
) -> Result<Json<Vec<PersonSummary>>, db::Error> {
    select_project_people(id, query, &mut db_conn)
        .await
        .map(Json)
}

async fn select_project_people(
    project_id: Uuid,
    PersonQuery {
        filter,
        limit,
        offset,
        order_by,
    }: PersonQuery,
    db_conn: &mut DbConnection,
) -> Result<Vec<PersonSummary>, db::Error> {
    let mut stmt = PersonSummary::query()
        .inner_join(project_people::table)
        .limit(limit)
        .offset(offset)
        .filter(project_people::project_id.eq(project_id))
        .filter(filter.to_boxed_filter())
        .into_boxed();

    for ordering in order_by.as_ref() {
        stmt = stmt.then_order_by(ordering);
    }

    Ok(stmt.load(db_conn).await?)
}
