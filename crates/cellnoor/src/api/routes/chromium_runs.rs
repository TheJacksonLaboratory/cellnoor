use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::chromium_run::{ChromiumRun, ChromiumRunQuery, SimpleChromiumRunQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::chromium_runs::{
        create::create_chromium_run, delete::delete_chromium_run, index::index_chromium_runs,
        show::show_chromium_run, update::update_chromium_run,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_chromium_run).get(index_chromium_runs_simple),
        )
        .api_route("/search", post(index_chromium_runs))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(show_chromium_run)
            .put(update_chromium_run)
            .delete(delete_chromium_run),
    )
}

async fn index_chromium_runs_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRun>>, Error> {
    index_chromium_runs(state, user, Json(ChromiumRunQuery::from_simple_query(q))).await
}
