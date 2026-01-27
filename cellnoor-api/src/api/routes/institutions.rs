use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Router,
    extract::State,
    handler::Handler,
    http::StatusCode,
    middleware::{map_request, map_response},
    response::Response,
};
use cellnoor_models::{
    institution::{Institution, InstitutionId, InstitutionQuery, NewInstitution},
    person::{PersonQuery, PersonSummary},
};
use tower::ServiceBuilder;

use crate::{
    api::middleware::{admin_required, creation_status_code},
    state::AppState,
};

pub mod create;
pub mod index;
// pub mod members;
pub mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    let creation_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(creation_status_code));

    let create_institution = create::create_institution.layer(creation_middleware);

    let router = ApiRouter::new()
        .api_route("/", post(create_institution).get(index::index_institutions))
        .api_route("/{id}", get(show::show_institution));

    router

    // router.merge(members::router())
}
