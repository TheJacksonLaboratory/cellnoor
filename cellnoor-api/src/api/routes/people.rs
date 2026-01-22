use axum::Router;
use axum_extra::routing::RouterExt;

use super::{ApiResponse, Root, handle_api_request};
use crate::state::AppState;

pub mod create;
pub mod fetch;
pub mod list;
pub mod update;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_person)
        .typed_get(fetch::fetch_person)
        .typed_get(list::list_people)
        .typed_patch(update::update_person)
}
