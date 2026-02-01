use aide::axum::{ApiRouter, routing::get};
use axum::Router;

use crate::state::AppState;

use index::index_multiplexing_tags;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_multiplexing_tags))
}
