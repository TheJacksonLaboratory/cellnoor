use crate::{api::request::nested_index, state::AppState};
use axum_extra::routing::Resource;
use cellnoor_models::{
    chromium_dataset::{ChromiumDatasetQuery, ChromiumDatasetSummary},
    project::ProjectId,
    specimen::{SpecimenQuery, SpecimenSummary},
};

use super::RESOURCE_NAME;

mod index;

pub(super) fn router() -> Resource<AppState> {
    Resource::named(&format!("{RESOURCE_NAME}/{{id}}/chromium-datasets"))
        .index(nested_index::<ProjectId, ChromiumDatasetQuery, Vec<ChromiumDatasetSummary>>)
}
