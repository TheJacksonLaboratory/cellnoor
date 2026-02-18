use axum::{
    Router,
    extract::DefaultBodyLimit,
    handler::Handler,
    routing::{get, post},
};
use metrics::{download::download_metrics_file, upload::upload_metrics_file};
use web_summaries::{download::download_web_summary, upload::upload_web_summary};

use crate::{admin_required_creation, state::AppState};

mod common;
mod metrics;
mod web_summaries;

pub(super) fn router() -> Router<AppState> {
    const BODY_LIMIT_128MB: usize = 128_000_000;

    let file_upload_layer =
        admin_required_creation!().layer(DefaultBodyLimit::max(BODY_LIMIT_128MB));

    Router::new()
        .route(
            "/metrics",
            post(upload_metrics_file).layer(file_upload_layer.clone()),
        )
        .route(
            "/metrics/{directory}/{filename}",
            get(download_metrics_file),
        )
        .route(
            "/web-summaries",
            post(upload_web_summary.layer(file_upload_layer)),
        )
        .route(
            "/web-summaries/{directory}/{filename}",
            get(download_web_summary),
        )
}
