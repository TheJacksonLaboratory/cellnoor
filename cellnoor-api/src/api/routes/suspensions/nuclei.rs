use aide::axum::{ApiRouter, routing::post};

use crate::state::AppState;

use create::create_nucleus_suspension;

mod create;
mod measurements;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", post(create_nucleus_suspension))
}
