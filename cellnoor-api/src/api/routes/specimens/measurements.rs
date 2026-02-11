use aide::axum::{ApiRouter, routing::get};
use axum::handler::Handler;
use create::create_specimen_measurement;
use index::index_specimen_measurements;

use crate::{admin_required_creation, state::AppState};

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(index_specimen_measurements)
            .post(create_specimen_measurement.layer(admin_required_creation!())),
    )
}
