use aide::axum::{
    ApiRouter,
    routing::{get, post},
};

use crate::{
    handlers::specimens::{create::create_specimen, index::index_specimens, show::show_specimen},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_specimen).get(simple_index_specimens))
        .api_route("/search", post(index_specimens))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_specimen))
}
