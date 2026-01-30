use aide::axum::{
    ApiRouter,
    routing::{get, post, post_with},
};

use crate::{admin_required_creation, state::AppState};

use index::index_suspensions;
use show::show_suspension;

pub(super) mod cells;
pub(super) mod index;
mod measurements;
pub(super) mod nuclei;
pub(super) mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(index_suspensions))
        .nest("/cells", cells::router().layer(admin_required_creation!()))
        .nest(
            "/nuclei",
            nuclei::router().layer(admin_required_creation!()),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_suspension))
        .nest("/measurements", measurements::router())
}
