use aide::axum::ApiRouter;

use crate::state::AppState;

mod create;
mod fetch;
mod list;
mod measurements;
mod suspensions;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/")
        .typed_post(create::create_suspension_pool)
        .nest("/cells", cells::router())
        .nest("/nuclei", nuclei::router())
        .typed_get(fetch::fetch_suspension_pool)
        .typed_get(list::list_suspension_pools)
        .typed_get(suspensions::list::list_suspensions)
        .typed_get(measurements::list::list_measurements)
}
