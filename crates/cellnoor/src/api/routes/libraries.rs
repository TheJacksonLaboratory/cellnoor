use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_library;
use index::index_libraries;
use show::show_library;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod index;
pub mod measurements;
pub mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_library.layer(admin_required_creation!())).get(index_libraries),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_library))
        .nest("/measurements", measurements::router())
}
