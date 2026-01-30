use crate::{admin_required_creation, state::AppState};
use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_institution;
use index::index_institutions;
use show::show_institution;

pub(super) mod create;
pub(super) mod index;
pub(super) mod members;
pub(super) mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_institution.layer(admin_required_creation!())).get(index_institutions),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_institution))
        .nest("/members", members::router())
}
