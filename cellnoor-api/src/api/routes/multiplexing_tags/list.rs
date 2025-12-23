use axum::{extract::State, http::StatusCode};
use cellnoor_models::multiplexing_tag::MultiplexingTag;
use cellnoor_schema::multiplexing_tags::dsl::*;
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn list_multiplexing_tags(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<MultiplexingTag>> {
    let items = inner_handler(state, user, ()).await?;
    Ok((StatusCode::OK, items))
}

impl db::Operation<Vec<MultiplexingTag>> for () {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<MultiplexingTag>, db::Error> {
        Ok(MultiplexingTag::query()
            .order_by((type_, tag_id))
            .load(db_conn)?)
    }
}
