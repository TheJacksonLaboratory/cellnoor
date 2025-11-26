use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod fetch;
mod list;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_suspension)
        .typed_get(fetch::fetch_suspension)
    // .typed_post(measurements::create::create_measurement)
    // .typed_get(measurements::list::list_measurements)
}
