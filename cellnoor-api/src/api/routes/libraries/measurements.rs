use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_library_measurement;
use index::index_library_measurements;

use crate::{admin_required_creation, state::AppState};

pub(super) mod create;
pub(super) mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        post(create_library_measurement.layer(admin_required_creation!()))
            .get(index_library_measurements),
    )
}
