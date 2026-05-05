use aide::axum::{
    ApiRouter,
    routing::{get, post},
};

use crate::{
    handlers::institutions::{
        create::create_institution, index::index_institutions, show::show_institution,
    },
    state::AppState,
};

pub(super) fn router<'a>() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_institution).get(index_institutions))
        .nest("/{id}", id_router())
}

fn id_router<'a>() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_institution))
}
