use aide::axum::{ApiRouter, routing::post};
use create::create_suspension_measurement;
use index::index_suspension_measurements;

use crate::state::AppState;

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        post(create_suspension_measurement).get(index_suspension_measurements),
    )
}
