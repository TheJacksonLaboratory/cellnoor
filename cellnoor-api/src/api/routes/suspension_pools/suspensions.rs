use aide::axum::{ApiRouter, routing::get};
use index::index_pooled_suspensions;

use crate::state::AppState;

pub(super) mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_pooled_suspensions))
}
