use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use cellnoor_models::institution::{Institution, InstitutionId};
use cellnoor_schema::institutions::dsl::id;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{
    api::{self, auth, routes::ApiResponse},
    db::{self, DbConnection},
    state::AppState,
};

pub async fn show_institution(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(institution_id): Path<InstitutionId>,
) -> Result<Json<Institution>, api::Error> {
    fetch_institution(institution_id, db_conn).await.map(Json)
}

pub async fn fetch_institution(
    institution_id: InstitutionId,
    mut db_conn: DbConnection,
) -> Result<Institution, api::Error> {
    Ok(Institution::query()
        .filter(id.eq(institution_id))
        .first(&mut db_conn)
        .await?)
}
