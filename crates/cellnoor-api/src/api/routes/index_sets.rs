use aide::axum::{ApiRouter, routing::post};

use crate::{
    handlers::index_sets::{create_dual_index_sets, create_single_index_sets},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/single", post(create_single_index_sets))
        .api_route("/dual", post(create_dual_index_sets))
}
