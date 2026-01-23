use crate::{
    api::request::{nested_create, nested_index},
    state::AppState,
};
use axum_extra::routing::Resource;
use cellnoor_models::{
    person::{PersonId, PersonQuery, PersonSummary},
    project::ProjectId,
    specimen::{SpecimenQuery, SpecimenSummary},
};

use super::RESOURCE_NAME;

mod create;
mod index;

pub(super) fn router() -> Resource<AppState> {
    Resource::named(&format!("{RESOURCE_NAME}/{{id}}/people"))
        .create(nested_create::<ProjectId, PersonId, ()>)
        .index(nested_index::<ProjectId, PersonQuery, Vec<PersonSummary>>)
}
