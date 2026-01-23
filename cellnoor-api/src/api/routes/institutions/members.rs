use axum_extra::routing::Resource;
use cellnoor_models::{
    institution::InstitutionId,
    person::{PersonQuery, PersonSummary},
};

use super::RESOURCE_NAME;
use crate::{api::request::nested_index, state::AppState};

mod index;

pub(super) fn router() -> Resource<AppState> {
    Resource::named(&format!("{RESOURCE_NAME}/{{id}}/members"))
        .index(nested_index::<InstitutionId, PersonQuery, Vec<PersonSummary>>)
}
