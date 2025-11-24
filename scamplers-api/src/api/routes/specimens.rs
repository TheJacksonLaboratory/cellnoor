use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod fetch;
mod list;
mod measurements;
mod update;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_specimen)
        .typed_get(fetch::fetch_specimen)
        .typed_get(list::list_specimens)
        .typed_post(measurements::create::create_measurement)
        .typed_get(measurements::list::list_measurements)
}
