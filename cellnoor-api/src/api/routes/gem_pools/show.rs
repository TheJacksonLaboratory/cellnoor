use axum::{
    Extension, Json,
    extract::{Path, State},
};
use cellnoor_models::{IdParameter, chromium_run::GemPool};
use cellnoor_schema::gem_pools;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    api::auth::AuthUser,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn show_gem_pool(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(_user): Extension<AuthUser>,
    Path(IdParameter { id }): Path<IdParameter>,
) -> Result<Json<GemPool>, db::Error> {
    select_gem_pool_by_id(id, &db_conn).await.map(Json)
}

pub async fn select_gem_pool_by_id(
    gem_pool_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<GemPool, db::Error> {
    Ok(GemPool::query()
        .filter(gem_pools::id.eq(gem_pool_id))
        .first(&mut db_conn)
        .await?)
}
