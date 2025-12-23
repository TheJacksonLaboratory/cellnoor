use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod fetch;
mod list;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_get(list::list_gems)
        .typed_get(fetch::fetch_gem_pool)
}
