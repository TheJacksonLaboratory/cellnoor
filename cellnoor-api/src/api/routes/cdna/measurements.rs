use aide::axum::{ApiRouter, routing::post};
use axum::handler::Handler;
use create::create_cdna_measurement;
pub use create::validate_electrophoretic_measurement;
use index::index_cdna_measurements;

use crate::{admin_required_creation, state::AppState};

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        post(create_cdna_measurement.layer(admin_required_creation!()))
            .get(index_cdna_measurements),
    )
}
