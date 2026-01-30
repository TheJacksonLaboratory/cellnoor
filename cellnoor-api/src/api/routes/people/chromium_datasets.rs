use crate::state::AppState;
use aide::axum::{ApiRouter, routing::get};

// mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(async || ()))
}
