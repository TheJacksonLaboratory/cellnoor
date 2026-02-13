use aide::axum::{ApiRouter, routing::get};
use index::index_people_staff_view;

use crate::state::AppState;

mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().route("/", get(index_people_staff_view))
}
