use aide::axum::{ApiRouter, routing::get};
use show::show_parsed_chromium_dataset_file;

use crate::state::AppState;

mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().route(
        "/parsed-files/{*path}",
        get(show_parsed_chromium_dataset_file),
    )
}
