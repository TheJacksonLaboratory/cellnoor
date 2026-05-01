use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use create::create_institution;

// use index::index_institutions;
// use show::show_institution;
use crate::state::AppState;

pub mod create;
// pub mod index;
// pub mod show;

pub(super) fn router<'a>() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", post(create_institution))
    // .nest("/{id}", id_router())
}

// fn id_router<'a>() -> ApiRouter<AppState> {
//     ApiRouter::new().api_route("/", get(show_institution))
// }
