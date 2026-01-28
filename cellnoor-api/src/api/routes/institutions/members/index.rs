use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use cellnoor_models::{
    IdParameter,
    institution::{self},
    person::{self, PersonFilter, PersonQuery, PersonSummary},
};
use diesel::prelude::*;
use diesel_async::AsyncPgConnection;
use uuid::Uuid;

use crate::{
    api::{
        self,
        auth::{self},
        extract::JsonQuery,
        routes::people::index::index_people,
    },
    db::{self, DbConnection, ToBoxedFilter},
    state::AppState,
};

pub async fn index_institution_members(
    state: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    mut query: JsonQuery<PersonQuery>,
) -> Result<Json<Vec<PersonSummary>>, db::Error> {
    query.query.filter.institution_ids = Some(vec![id]);

    index_people(state, db_conn, query).await
}
