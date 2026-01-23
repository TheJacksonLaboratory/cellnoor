use crate::{
    api::{extract::PathAndQuery, request::nested_index},
    state::AppState,
};
use axum_extra::routing::Resource;
use cellnoor_models::{
    person::PersonId,
    project::{Project, ProjectQuery},
    specimen::{SpecimenQuery, SpecimenSummary},
};

use super::RESOURCE_NAME;

mod index;

pub(super) fn router() -> Resource<AppState> {
    Resource::named(&format!("{RESOURCE_NAME}/{{id}}/projects"))
        .index(nested_index::<PersonId, ProjectQuery, Vec<Project>>)
}
