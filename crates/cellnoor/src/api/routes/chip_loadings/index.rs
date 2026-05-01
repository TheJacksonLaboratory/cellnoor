use axum::{Json, extract::State};
use cellnoor_models::chip_loading::ChipLoading;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn index_chip_loadings(
    _: State<AppState>,
    db_conn: DbConnection,
) -> Result<Json<Vec<ChipLoading>>, db::Error> {
    select_chip_loadings(&db_conn).await.map(Json)
}

// We can improve this route in the future by providing a way to filter and
// maybe joining, but for now, we just want to see all the IDs
pub async fn select_chip_loadings(
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Vec<ChipLoading>, db::Error> {
    let stmt = ChipLoading::query();

    Ok(stmt.load(&mut db_conn).await?)
}
