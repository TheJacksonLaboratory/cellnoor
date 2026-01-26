use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{Router, extract::State, http::StatusCode};
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

pub(super) fn router() -> ApiRouter<AppState> {
    let router = ApiRouter::new()
        .api_route(
            "/",
            post(create::<InstitutionCreation, Institution>)
                .get(index::<InstitutionQuery, Vec<Institution>>),
        )
        .api_route("/{id}", get(show::<InstitutionId, Institution>));

    router.merge(members::router())
}
