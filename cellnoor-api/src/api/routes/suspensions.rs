use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod cells;
mod create;
mod fetch;
mod list;
mod measurements;
mod nuclei;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .nest("/cells", cells::router())
        .nest("/nuclei", nuclei::router())
        .typed_get(fetch::fetch_suspension)
        .typed_get(list::list_suspensions)
        .typed_get(measurements::list::list_measurements)
}
