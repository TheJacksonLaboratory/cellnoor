use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    person::{PersonQuery, PersonSummary},
};
use diesel::prelude::*;

use crate::{
    api::{extract::JsonQuery, routes::people::index::index_people},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_members(
    state: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    mut query: JsonQuery<PersonQuery>,
) -> Result<Json<Vec<PersonSummary>>, db::Error> {
    query.q.filter.institution_ids = Some(vec![id]);

    index_people(state, db_conn, query).await
}
