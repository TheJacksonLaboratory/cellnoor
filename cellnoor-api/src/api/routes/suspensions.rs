use aide::axum::{
    ApiRouter,
    routing::{get, post, post_with},
};
use axum::handler::Handler;

use crate::{admin_required_creation, state::AppState};

use create::create_suspension;
use index::index_suspensions;
use show::show_suspension;

pub(super) mod create;
pub(super) mod index;
mod measurements;
pub(super) mod show;

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
