use aide::axum::{ApiRouter, routing::get};
use index::index_project_specimens;

use crate::state::AppState;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(index_project_specimens))
}
