use axum::{Router, extract::State, http::StatusCode};
use axum_extra::routing::{Resource, RouterExt};
use cellnoor_models::{
    institution::{Institution, InstitutionCreation, InstitutionId, InstitutionQuery},
    person::{PersonQuery, PersonSummary},
};

use crate::{
    api::{
        AuthenticatedUser,
        extract::{Json, Path, PathAndJson, PathAndQuery, QsQuery},
        request::{create, index, nested_index, show},
    },
    state::AppState,
};

mod create;
mod index;
mod members;
mod show;

const RESOURCE_NAME: &str = "institutions";

pub(super) fn router() -> Router<AppState> {
    let router: Router<AppState> = Resource::named(RESOURCE_NAME)
        .create(create::<InstitutionCreation, Institution>)
        .show(show::<InstitutionId, Institution>)
        .index(index::<InstitutionQuery, Vec<Institution>>)
        .into();

    router.merge(members::router())
}
