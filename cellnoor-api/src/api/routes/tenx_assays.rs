use aide::axum::{ApiRouter, routing::get};

use crate::state::AppState;

use index::index_tenx_assays;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_tenx_assays))
}
