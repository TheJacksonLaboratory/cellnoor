use aide::axum::{
    ApiRouter,
    routing::{get, post},
};

use crate::{
    handlers::people::{create::create_person, index::index_people, show::show_person},
    state::AppState,
};

pub(super) fn router<'a>() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_person).get(index_people))
        .nest("/{id}", id_router())
}

fn id_router<'a>() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_person))
}
