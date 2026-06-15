use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::chromium_run::{
    ChromiumRunCompact, ChromiumRunDetailed, ChromiumRunQuery, SimpleChromiumRunQuery,
};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::chromium_runs::{
        create_chromium_run, delete_chromium_run, index_chromium_runs,
        index_chromium_runs_detailed, show_chromium_run, update_chromium_run,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_chromium_run).get(index_chromium_runs_simple),
        )
        .api_route("/detailed", get(index_chromium_runs_detailed_simple))
        .api_route("/search", post(index_chromium_runs))
        .api_route("/search/detailed", post(index_chromium_runs_detailed))
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
) -> Result<Json<Vec<ChromiumRunCompact>>, Error> {
    index_chromium_runs(state, user, Json(ChromiumRunQuery::from_simple_query(q))).await
}

async fn index_chromium_runs_detailed_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleChromiumRunQuery>,
) -> Result<Json<Vec<ChromiumRunDetailed>>, Error> {
    index_chromium_runs_detailed(state, user, Json(ChromiumRunQuery::from_simple_query(q))).await
}
