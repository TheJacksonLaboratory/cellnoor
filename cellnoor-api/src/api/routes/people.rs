use crate::{
    api::{
        extract::{Json, Path, PathAndJson, QsQuery},
        request::{CREATED, OK, handle_api_request},
    },
    state::AppState,
};
use axum::Router;
use axum_extra::routing::Resource;
use cellnoor_models::person::{
    Person, PersonCreation, PersonId, PersonQuery, PersonSummary, PersonUpdate,
};

pub mod create;
pub mod fetch;
pub mod list;
pub mod update;

pub(super) fn router() -> Resource<AppState> {
    Resource::named("people")
        .create(handle_api_request::<Json<PersonCreation>, Person, CREATED>)
        .show(handle_api_request::<Path<PersonId>, Person, OK>)
        .index(handle_api_request::<QsQuery<PersonQuery>, Vec<PersonSummary>, OK>)
        .update(handle_api_request::<PathAndJson<PersonId, PersonUpdate>, Person, OK>)
}
