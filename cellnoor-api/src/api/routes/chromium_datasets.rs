use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;
use create::create_chromium_dataset;
use index::index_chromium_datasets;
use show::show_chromium_dataset;

use crate::{admin_required_creation, state::AppState};

pub mod create;
pub mod files;
pub mod index;
pub mod libraries;
pub mod show;
pub mod specimens;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_chromium_dataset.layer(admin_required_creation!()))
                .get(index_chromium_datasets),
        )
        .nest("/{dataset_id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_chromium_dataset))
        .nest("/specimens", specimens::router())
        .nest("/libraries", libraries::router())
        .merge(files::router())
}
