use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    person::{PersonQuery, PersonSummary},
};

use crate::{
    api::{extract::AuthJsonQuery, routes::people::index::index_people},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn index_institution_members(
    state: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    mut query: AuthJsonQuery<PersonQuery>,
) -> Result<Json<Vec<PersonSummary>>, db::Error> {
    query.q.filter.institution_ids = Some(vec![id]);

    index_people(state, db_conn, query).await
}
