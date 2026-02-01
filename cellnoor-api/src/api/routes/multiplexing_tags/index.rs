use axum::{Json, extract::State};
use cellnoor_models::multiplexing_tag::MultiplexingTag;
use cellnoor_schema::multiplexing_tags::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn index_multiplexing_tags(
    _: State<AppState>,
    mut db_conn: DbConnection,
) -> Result<Json<Vec<MultiplexingTag>>, db::Error> {
    select_multiplexing_tags(&mut db_conn).await.map(Json)
}

pub async fn select_multiplexing_tags(
    db_conn: &mut DbConnection,
) -> Result<Vec<MultiplexingTag>, db::Error> {
    Ok(MultiplexingTag::query()
        .order_by((type_, tag_id))
        .load(db_conn)
        .await?)
}
