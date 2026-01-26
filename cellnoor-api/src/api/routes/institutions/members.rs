use aide::axum::{ApiRouter, routing::get};
use cellnoor_models::{
    institution::InstitutionId,
    person::{PersonQuery, PersonSummary},
};

use super::RESOURCE_NAME;
use crate::{api::request::nested_index, state::AppState};

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/{id}/members",
        get(nested_index::<InstitutionId, PersonQuery, Vec<PersonSummary>>),
    )
}
