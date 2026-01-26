use crate::{
    api::{extract::PathAndQuery, request::nested_index},
    state::AppState,
};
use aide::axum::{ApiRouter, routing::get};
use cellnoor_models::{
    person::PersonId,
    specimen::{SpecimenQuery, SpecimenSummary},
};

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().route(
        "/{id}/specimens",
        get(nested_index::<PersonId, SpecimenQuery, Vec<SpecimenSummary>>),
    )
}
