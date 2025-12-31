use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod list;
mod members;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_lab)
        .typed_get(list::list_labs)
}
