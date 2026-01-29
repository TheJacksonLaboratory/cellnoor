use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use tower::ServiceBuilder;

use crate::{
    api::middleware::{admin_required, created_status_code},
    state::AppState,
};
use create::create_institution;
use index::index_institutions;
use index_members::index_members;
use show::show_institution;

pub mod create;
pub mod index;
pub mod index_members;
pub mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    let id_router = ApiRouter::new()
        .api_route("/", get(show_institution))
        .api_route("/members", get(index_members));

    let creation_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(created_status_code));

    ApiRouter::new()
        .api_route(
            "/",
            post(create_institution.layer(creation_middleware)).get(index_institutions),
        )
        .nest("/{id}", id_router)
}
