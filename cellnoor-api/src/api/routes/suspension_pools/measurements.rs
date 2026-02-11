use aide::axum::{ApiRouter, routing::post};
use axum::handler::Handler;
use create::create_suspension_pool_measurement;
use index::index_measurements;

use crate::{admin_required_creation, state::AppState};

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        post(create_suspension_pool_measurement.layer(admin_required_creation!()))
            .get(index_measurements),
    )
}
