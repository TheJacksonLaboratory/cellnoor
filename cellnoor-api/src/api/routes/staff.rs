use aide::axum::ApiRouter;

use crate::{api::middleware::staff_required, state::AppState};

mod people;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .nest("/people", people::router())
        .layer(axum::middleware::from_fn(staff_required))
}
