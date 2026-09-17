use aide::axum::{ApiRouter, routing::get};

use crate::{
    handlers::multiplexing_tags::{create_multiplexing_tag, index_multiplexing_tags},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(index_multiplexing_tags).post(create_multiplexing_tag),
    )
}
