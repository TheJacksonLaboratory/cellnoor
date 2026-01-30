use crate::{admin_required_creation, state::AppState};
use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_specimen;
use index::index_specimens;
use show::show_specimen;

pub(super) mod chromium_datasets;
pub(super) mod create;
pub(super) mod index;
pub(super) mod measurements;
pub(super) mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_specimen.layer(admin_required_creation!())).get(index_specimens),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_specimen))
        .nest("/measurements", measurements::router())
        .nest("/chromium-datasets", chromium_datasets::router())
}
