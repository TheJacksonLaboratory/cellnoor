use aide::axum::{
    ApiRouter,
    routing::{get, get_with, post, post_with},
};
use axum::handler::Handler;
use tower::ServiceBuilder;

use crate::{
    api::{
        docs::db_and_auth_error_docs,
        middleware::{admin_required, creation_status_code},
    },
    state::AppState,
};

pub mod create;
pub mod index;
pub mod members;
pub mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    let creation_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(creation_status_code));

    let create_institution = create::create_institution.layer(creation_middleware);

    let root_router = ApiRouter::new()
        .api_route(
            "/",
            post_with(create_institution, db_and_auth_error_docs)
                .get_with(index::index_institutions, db_and_auth_error_docs),
        )
        .api_route(
            "/{id}",
            get_with(show::show_institution, db_and_auth_error_docs),
        );

    root_router.merge(members::router())
}
