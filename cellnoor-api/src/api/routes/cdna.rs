use crate::{admin_required_creation, state::AppState};
use aide::axum::{
    ApiRouter,
    routing::{get, post},
};

use create::create_cdna;
use index::list_cdna;
use show::fetch_cdna;

mod create;
mod index;
mod measurements;
mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_cdna.layer(admin_required_creation!())).get(list_cdna),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(fetch_cdna))
        .api_route("/measurements", measurements::router())
}
