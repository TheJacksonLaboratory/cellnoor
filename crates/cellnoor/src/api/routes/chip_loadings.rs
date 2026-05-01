use aide::axum::{ApiRouter, routing::get};
use axum::handler::Handler;
use index::index_chip_loadings;

use crate::{api::middleware::staff_required, state::AppState};

pub mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(index_chip_loadings.layer(axum::middleware::from_fn(staff_required))),
    )
}
