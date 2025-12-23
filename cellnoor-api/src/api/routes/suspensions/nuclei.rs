use axum::{Router, routing::post};
use axum_extra::routing::{RouterExt, TypedPath};
use cellnoor_models::suspension::SuspensionIdMeasurements;

use crate::state::AppState;

mod create;
mod measurements;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_nucleus_suspension)
        .route(
            SuspensionIdMeasurements::PATH,
            post(measurements::create_nucleus_suspension_measurement),
        )
}
