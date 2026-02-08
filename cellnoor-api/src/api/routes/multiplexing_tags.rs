use aide::axum::{ApiRouter, routing::get};
use index::index_multiplexing_tags;

use crate::state::AppState;

pub mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_multiplexing_tags))
}
