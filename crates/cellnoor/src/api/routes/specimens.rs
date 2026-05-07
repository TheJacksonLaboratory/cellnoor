use aide::axum::{ApiRouter, routing::post};

use crate::{
    handlers::specimens::measurements::create::create_specimen_measurement, state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/measurements", post(create_specimen_measurement))
}
