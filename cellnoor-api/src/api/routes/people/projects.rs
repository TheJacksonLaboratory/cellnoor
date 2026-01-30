use aide::axum::{ApiRouter, routing::get};
use index::index_person_projects;

use crate::state::AppState;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().route("/", get(index_person_projects))
}
