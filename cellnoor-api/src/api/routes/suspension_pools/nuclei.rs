use axum::{Router, routing::post};
use axum_extra::routing::TypedPath;
use cellnoor_models::suspension_pool::SuspensionPoolIdMeasurements;

use crate::state::AppState;

mod measurements;

pub(super) fn router() -> Router<AppState> {
    Router::new().route(
        SuspensionPoolIdMeasurements::PATH,
        post(measurements::create_nucleus_suspension_pool_measurement),
    )
}
