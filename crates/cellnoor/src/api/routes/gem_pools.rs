use aide::axum::{ApiRouter, routing::get};
use index::index_gem_pools;
use show::show_gem_pool;

use crate::state::AppState;

pub mod index;
pub mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(index_gem_pools))
        .api_route("/{id}", get(show_gem_pool))
}
