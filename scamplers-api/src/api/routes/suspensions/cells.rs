use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod measurements;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_cell_suspension)
        .typed_post(measurements::create_cell_suspension_measurement)
}
