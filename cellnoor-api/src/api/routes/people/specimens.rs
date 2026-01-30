use aide::axum::{ApiRouter, routing::get};

use crate::state::AppState;

use index::index_person_specimens;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().route("/", get(index_person_specimens))
}
