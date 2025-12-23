use axum::{Router, routing::post};
use axum_extra::routing::{RouterExt, TypedPath};
use cellnoor_models::cdna::CdnaIdMeasurements;

use crate::state::AppState;

mod create;
mod fetch;
mod list;
mod measurements;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_cdna)
        .typed_get(fetch::fetch_cdna)
        .typed_get(list::list_cdna)
        .route(
            CdnaIdMeasurements::PATH,
            post(measurements::create::create_measurement),
        )
        .typed_get(measurements::list::list_measurements)
}
