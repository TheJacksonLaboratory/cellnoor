use axum::{Router, handler::Handler, routing::get};
use backup::fetch_db_backup;

use crate::{api::middleware::admin_required, state::AppState};

pub mod backup;

pub(super) fn router() -> Router<AppState> {
    Router::new().route(
        "/backup",
        get(fetch_db_backup.layer(axum::middleware::from_fn(admin_required))),
    )
}
