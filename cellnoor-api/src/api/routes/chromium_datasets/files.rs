use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use upload::upload_file;

use crate::{
    admin_required_creation,
    api::routes::chromium_datasets::files::download::download_chromium_dataset_file,
    state::AppState,
};

mod download;
mod upload;

pub(super) fn router() -> Router<AppState> {
    const WEB_SUMMARY_FILE_SIZE: usize = 16 * 1_000_000; // 16 MB
    const MAX_N_SAMPLES: usize = 384;

    // Each sample has its own web_summary.html
    const BODY_SIZE_LIMIT: usize = WEB_SUMMARY_FILE_SIZE * MAX_N_SAMPLES;

    let file_upload_layer =
        admin_required_creation!().layer(DefaultBodyLimit::max(BODY_SIZE_LIMIT));

    Router::new()
        .route("/files/", post(upload_file).layer(file_upload_layer))
        .route("/files/{*path}", get(download_chromium_dataset_file))
}
