use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod fetch;
mod list;
mod measurements;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_library)
        .typed_get(fetch::fetch_library)
        .typed_get(list::list_libraries)
        .typed_post(measurements::create::create_measurement)
        .typed_get(measurements::list::list_measurements)
}
