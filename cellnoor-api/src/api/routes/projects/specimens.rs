use crate::state::AppState;
use aide::axum::{ApiRouter, routing::get};

use index::index_project_specimens;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_project_specimens))
}
