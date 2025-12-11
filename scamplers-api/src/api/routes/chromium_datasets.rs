use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

pub(crate) mod common;
mod create;
mod fetch;
mod libraries;
mod list;
mod read;
mod specimens;
mod web_summaries;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_chromium_dataset)
        .typed_post(web_summaries::upload::upload_web_summary)
        .typed_get(fetch::fetch_chromium_dataset)
        .typed_get(list::list_chromium_datasets)
        .typed_get(specimens::list::list_specimens)
        .typed_get(libraries::list::list_libraries)
        .typed_get(web_summaries::fetch::fetch_web_summary)
}
