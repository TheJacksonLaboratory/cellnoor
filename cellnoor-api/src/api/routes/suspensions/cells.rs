use aide::axum::ApiRouter;
use axum::{Router, routing::post};

use crate::state::AppState;

mod create;
mod measurements;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/")
        .typed_post(create::create_cell_suspension)
        .route(
            SuspensionIdMeasurements::PATH,
            post(measurements::create_cell_suspension_measurement),
        )
}
