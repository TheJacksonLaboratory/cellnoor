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
    const BODY_LIMIT_128MB: usize = 128_000_000;

    let file_upload_layer =
        admin_required_creation!().layer(DefaultBodyLimit::max(BODY_LIMIT_128MB));

    Router::new()
        .route("/files/", post(upload_file).layer(file_upload_layer))
        .route("/files/{*path}", get(download_chromium_dataset_file))
}
