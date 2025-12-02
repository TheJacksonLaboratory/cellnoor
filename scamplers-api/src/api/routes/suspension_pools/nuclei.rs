use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod measurements;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_nucleus_suspension_pool)
        .typed_post(measurements::create_nucleus_suspension_pool_measurement)
}
