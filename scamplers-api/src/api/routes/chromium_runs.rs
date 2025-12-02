use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod fetch;
mod list;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_chromium_run)
        .typed_get(fetch::fetch_chromium_run)
        .typed_get(list::list_chromium_runs)
}
