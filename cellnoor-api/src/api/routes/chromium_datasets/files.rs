use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use upload::upload_files;

use crate::{
    admin_required_creation,
    api::routes::chromium_datasets::files::download::download_chromium_dataset_file,
    state::AppState,
};

mod download;
mod upload;

// This is an overestimate because the biggest I've seen is 9 MiB but you know
// the adage
const WEB_SUMMARY_FILE_SIZE: usize = 16 * 1024 * 1024; // 16 MiB
// Also super unlikely that we'll receive a dataset of 384 multiplexed samples
const MAX_N_SAMPLES: usize = 384;

pub(super) fn router() -> Router<AppState> {
    const BODY_SIZE_LIMIT: usize = WEB_SUMMARY_FILE_SIZE * MAX_N_SAMPLES;

    let file_upload_layer =
        admin_required_creation!().layer(DefaultBodyLimit::max(BODY_SIZE_LIMIT));

    Router::new()
        .route("/raw-files", post(upload_files).layer(file_upload_layer))
        .route("/raw-files/{*path}", get(download_chromium_dataset_file))
}
