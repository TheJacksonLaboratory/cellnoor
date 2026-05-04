use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_suspension;
use index::index_suspensions;
use show::show_suspension;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod index;
pub mod measurements;
pub mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_suspension.layer(admin_required_creation!())).get(index_suspensions),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_suspension))
        .nest("/measurements", measurements::router())
}
