use crate::{
    api::{extract::PathAndQuery, request::nested_index},
    state::AppState,
};
use aide::axum::{ApiRouter, routing::get};
use cellnoor_models::{
    chromium_dataset::{ChromiumDatasetQuery, ChromiumDatasetSummary},
    person::PersonId,
    specimen::{SpecimenQuery, SpecimenSummary},
};

use super::RESOURCE_NAME;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/{id}/chromium-datasets",
        get(nested_index::<PersonId, ChromiumDatasetQuery, Vec<ChromiumDatasetSummary>>),
    )
}
