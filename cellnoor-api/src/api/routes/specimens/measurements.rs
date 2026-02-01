use aide::axum::{ApiRouter, routing::get};
use axum::{Json, handler::Handler, http::StatusCode};

use crate::{admin_required_creation, state::AppState};

use create::create_specimen_measurement;
use index::index_specimen_measurements;

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(index_specimen_measurements)
            .post(create_specimen_measurement.layer(admin_required_creation!())),
    )
}
