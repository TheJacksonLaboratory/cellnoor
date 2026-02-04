use aide::axum::{ApiRouter, routing::get};
use index::index_specimen_chromium_datasets;

use crate::state::AppState;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_specimen_chromium_datasets))
}
