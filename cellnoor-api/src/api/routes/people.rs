use axum::Router;
use axum_extra::routing::RouterExt;

use super::{ApiResponse, Root, inner_handler};
use crate::state::AppState;

mod create;
mod fetch;
mod list;
mod update;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_person)
        .typed_get(fetch::fetch_person)
        .typed_get(list::list_people)
        .typed_patch(update::update_person)
}
