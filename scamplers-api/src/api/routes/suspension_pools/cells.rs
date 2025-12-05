use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod measurements;

pub(super) fn router() -> Router<AppState> {
    Router::new().typed_post(measurements::create_cell_suspension_pool_measurement)
}
