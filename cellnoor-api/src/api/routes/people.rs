use crate::{
    api::{
        extract::{Json, Path, PathAndJson, PathAndQuery, QsQuery},
        request::{create, index, nested_index, show, update},
    },
    state::AppState,
};
use axum::Router;
use axum_extra::routing::Resource;
use cellnoor_models::{
    person::{Person, PersonCreation, PersonId, PersonQuery, PersonSummary, PersonUpdate},
    specimen::{SpecimenQuery, SpecimenSummary},
};

pub use create::Error as CreatePersonError;

mod chromium_datasets;
mod create;
mod index;
mod projects;
mod show;
mod specimens;
mod update;

const RESOURCE_NAME: &str = "people";

pub(super) fn router() -> Router<AppState> {
    let root_router: Router<AppState> = Resource::named(RESOURCE_NAME)
        .create(create::<PersonCreation, Person>)
        .show(show::<PersonId, Person>)
        .index(index::<PersonQuery, Vec<PersonSummary>>)
        .update(update::<PersonId, PersonUpdate, Person>)
        .into();

    root_router
        .merge(projects::router())
        .merge(specimens::router())
        .merge(chromium_datasets::router())
}
