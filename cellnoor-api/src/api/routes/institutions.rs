use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{Router, extract::State, handler::Handler, http::StatusCode};
use cellnoor_models::{
    institution::{Institution, InstitutionId, InstitutionQuery, NewInstitution},
    person::{PersonQuery, PersonSummary},
};

use crate::{api::auth::admin_required, state::AppState};

pub mod create;
pub mod index;
// pub mod members;
pub mod show;

const RESOURCE_NAME: &str = "institutions";

pub(super) fn router() -> ApiRouter<AppState> {
    let router = ApiRouter::new()
        .api_route(
            "/",
            post(create::create_institution.layer(axum::middleware::from_fn(admin_required)))
                .get(index::index_institutions),
        )
        .api_route("/{id}", get(show::show_institution));

    router

    // router.merge(members::router())
}
