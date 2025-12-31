use axum::{Router, routing::post};
use axum_extra::routing::{RouterExt, TypedPath};
use cellnoor_models::specimen::SpecimenIdMeasurements;

use crate::state::AppState;

mod chromium_datasets;
mod create;
mod fetch;
mod list;
mod measurements;
mod update;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_specimen)
        .typed_get(fetch::fetch_specimen)
        .typed_get(list::list_specimens)
        .route(
            SpecimenIdMeasurements::PATH,
            post(measurements::create::create_measurement),
        )
        .typed_get(measurements::list::list_measurements)
        .typed_get(chromium_datasets::list::list_chromium_datasets)
}
