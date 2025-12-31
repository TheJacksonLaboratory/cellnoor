use axum::{Router, routing::post};
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod libraries;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_sequencing_run)
        .route(
            "/libraries",
            post(libraries::add_to_sequencing_run::add_libraries_to_sequencing_run),
        )
}
