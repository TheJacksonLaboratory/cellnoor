use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::handler::Handler;

use crate::{admin_required_creation, state::AppState};

use create::create_chromium_run;
use index::index_chromium_runs;
use show::show_chromium_run;

mod create;
mod index;
mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_chromium_run.layer(admin_required_creation!())).get(index_chromium_runs),
        )
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_chromium_run))
}
