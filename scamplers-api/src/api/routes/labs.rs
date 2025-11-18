use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod members;
mod read;

pub(super) fn router() -> Router<AppState> {
    Router::new().typed_post(create::create_lab)
}
