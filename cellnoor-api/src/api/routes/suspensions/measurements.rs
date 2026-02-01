use aide::axum::{ApiRouter, routing::post};

use crate::state::AppState;

use create::create_suspension_measurement;
use index::index_suspension_measurements;

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        post(create_suspension_measurement).get(index_suspension_measurements),
    )
}
