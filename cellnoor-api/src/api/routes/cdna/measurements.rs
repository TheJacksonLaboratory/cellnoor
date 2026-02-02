use aide::axum::{ApiRouter, routing::post};
use axum::handler::Handler;

use crate::{admin_required_creation, state::AppState};

use create::create_cdna_measurement;
use index::index_measurements;

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        post(create_cdna_measurement.layer(admin_required_creation!())).get(index_measurements),
    )
}
