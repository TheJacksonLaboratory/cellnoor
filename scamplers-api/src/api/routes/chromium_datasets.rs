use axum::{Router, extract::DefaultBodyLimit, handler::Handler};
use axum_extra::routing::RouterExt;
pub(crate) use list::chromium_datasets_to_all_specimens;

use crate::state::AppState;

mod create;
mod fetch;
mod files;
mod libraries;
mod list;
mod read;
mod specimens;

const ROUGHLY_16MB: usize = 2usize.pow(24);

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_chromium_dataset)
        .typed_post(files::metrics::upload::upload_metrics_file)
        .typed_post(
            files::web_summaries::upload::upload_web_summary
                .layer(DefaultBodyLimit::max(ROUGHLY_16MB)),
        )
        .typed_get(fetch::fetch_chromium_dataset)
        .typed_get(list::list_chromium_datasets)
        .typed_get(specimens::list::list_specimens)
        .typed_get(libraries::list::list_libraries)
        .typed_get(files::metrics::fetch::fetch_metrics_file)
        .typed_get(files::web_summaries::fetch::fetch_web_summary)
}
