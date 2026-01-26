use crate::{
    api::{extract::PathAndQuery, request::nested_index},
    state::AppState,
};
use aide::axum::{ApiRouter, routing::get};
use cellnoor_models::{
    person::PersonId,
    project::{Project, ProjectQuery},
    specimen::{SpecimenQuery, SpecimenSummary},
};

use super::RESOURCE_NAME;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/{id}/projects",
        get(nested_index::<PersonId, ProjectQuery, Vec<Project>>),
    )
}
