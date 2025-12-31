use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod cells;
mod create;
mod fetch;
mod list;
mod measurements;
mod nuclei;
mod suspensions;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_suspension_pool)
        .nest("/cells", cells::router())
        .nest("/nuclei", nuclei::router())
        .typed_get(fetch::fetch_suspension_pool)
        .typed_get(list::list_suspension_pools)
        .typed_get(suspensions::list::list_suspensions)
        .typed_get(measurements::list::list_measurements)
}
