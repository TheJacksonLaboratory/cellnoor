use crate::{api::request::nested_index, state::AppState};
use axum_extra::routing::Resource;
use cellnoor_models::{
    project::ProjectId,
    specimen::{SpecimenQuery, SpecimenSummary},
};

use super::RESOURCE_NAME;

mod index;

pub(super) fn router() -> Resource<AppState> {
    Resource::named(&format!("{RESOURCE_NAME}/{{id}}/specimens"))
        .index(nested_index::<ProjectId, SpecimenQuery, Vec<SpecimenSummary>>)
}
