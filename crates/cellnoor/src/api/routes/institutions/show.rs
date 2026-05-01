use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, institution::Institution};
use cellnoor_schema::institutions;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub async fn show_institution(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<Institution>, db::Error> {
    select_institution_by_id(id, db_conn).await.map(Json)
}

pub async fn select_institution_by_id(
    institution_id: Uuid,
    mut db_conn: DbConnection,
) -> Result<Institution, db::Error> {
    Ok(Institution::query()
        .filter(institutions::id.eq(institution_id))
        .first(&mut db_conn)
        .await?)
}
