use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_institution;
use index::index_institutions;
use show::show_institution;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod index;
pub mod members;
pub mod show;

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
