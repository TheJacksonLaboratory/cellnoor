use axum::Router;
use axum_extra::routing::RouterExt;

use crate::state::AppState;

mod create;
mod fetch;
mod list;
mod members;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .typed_post(create::create_institution)
        .typed_get(fetch::fetch_institution)
        .typed_get(list::list_institutions)
        .typed_get(members::list::list_members)
}
