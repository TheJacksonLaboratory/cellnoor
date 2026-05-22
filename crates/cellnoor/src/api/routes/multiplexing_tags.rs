use aide::axum::{
    ApiRouter,
    routing::{delete, get},
};

use crate::{
    handlers::multiplexing_tags::{
        create::create_multiplexing_tag, delete::delete_multiplexing_tag,
        index::index_multiplexing_tags,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get(index_multiplexing_tags).post(create_multiplexing_tag),
        )
        .api_route("/{id}", delete(delete_multiplexing_tag))
}
