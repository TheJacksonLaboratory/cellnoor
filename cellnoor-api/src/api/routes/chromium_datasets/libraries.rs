use aide::axum::{ApiRouter, routing::get};
use index::index_chromium_dataset_libraries;

use crate::state::AppState;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_chromium_dataset_libraries))
}
