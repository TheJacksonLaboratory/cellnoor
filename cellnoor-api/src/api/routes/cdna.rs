use crate::{admin_required_creation, state::AppState};
use aide::axum::{
    ApiRouter,
    routing::{get, post},
};

use axum::handler::Handler;
use create::create_cdna;
use index::index_cdna;
use show::show_cdna;

mod create;
mod index;
mod measurements;
mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_cdna.layer(admin_required_creation!())).get(index_cdna),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_cdna))
        .nest("/measurements", measurements::router())
}
