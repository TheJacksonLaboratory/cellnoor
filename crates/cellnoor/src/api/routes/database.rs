use axum::{Router, handler::Handler, routing::get};
use dump::dump_database;

use crate::{api::middleware::admin_required, state::AppState};

pub mod dump;

pub(super) fn router() -> Router<AppState> {
    Router::new().route(
        "/dump",
        get(dump_database.layer(axum::middleware::from_fn(admin_required))),
    )
}
