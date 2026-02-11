use axum::{
    Router,
    extract::DefaultBodyLimit,
    handler::Handler,
    routing::{get, post},
};
use metrics::{download::download_metrics_file, upload::upload_metrics_file};
use tower::ServiceBuilder;
use web_summaries::{download::download_web_summary, upload::upload_web_summary};

use crate::{
    api::middleware::{admin_required, created_status_code},
    state::AppState,
};

mod common;
mod metrics;
mod web_summaries;

pub(super) fn router() -> Router<AppState> {
    const ROUGHLY_16MB: usize = 2usize.pow(24);

    let file_upload_layer = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(created_status_code))
        .layer(DefaultBodyLimit::max(ROUGHLY_16MB));

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
