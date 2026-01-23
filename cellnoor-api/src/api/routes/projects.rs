use axum::Router;
use axum_extra::routing::{Resource, RouterExt};
use cellnoor_models::project::{Project, ProjectCreation, ProjectId, ProjectQuery};

use crate::{
    api::{
        extract::{Json, Path, QsQuery},
        request::{create, index, nested_index, show},
    },
    state::AppState,
};

mod chromium_datasets;
mod create;
mod index;
mod people;
mod show;
mod specimens;

const RESOURCE_NAME: &str = "projects";

pub(super) fn router() -> Router<AppState> {
    let router: Router<AppState> = Resource::named(RESOURCE_NAME)
        .create(create::<ProjectCreation, Project>)
        .show(show::<ProjectId, Project>)
        .index(index::<ProjectQuery, Vec<Project>>)
        .into();

    router
        .merge(people::router())
        .merge(specimens::router())
        .merge(chromium_datasets::router())
}
