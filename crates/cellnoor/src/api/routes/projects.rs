use aide::axum::{
    ApiRouter,
    routing::{get, post},
};

use crate::{
    handlers::projects::{create::create_project, index::index_projects, show::show_project},
    state::AppState,
};

pub(super) fn router<'a>() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_project).get(index_projects))
        .nest("/{id}", id_router())
}

fn id_router<'a>() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_project))
}
