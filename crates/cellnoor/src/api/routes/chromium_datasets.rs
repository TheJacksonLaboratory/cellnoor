use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::chromium_dataset::{
    ChromiumDatasetCompact, ChromiumDatasetQuery, SimpleChromiumDatasetQuery,
};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::chromium_datasets::{
        create_chromium_dataset, delete_chromium_dataset, index_chromium_datasets,
        index_chromium_datasets_detailed, show_chromium_dataset, update_chromium_dataset,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_chromium_dataset).get(index_chromium_datasets_simple),
        )
        .api_route("/search", post(index_chromium_datasets))
        .api_route("/search/detailed", post(index_chromium_datasets_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(show_chromium_dataset)
            .put(update_chromium_dataset)
            .delete(delete_chromium_dataset),
    )
}

async fn index_chromium_datasets_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleChromiumDatasetQuery>,
) -> Result<Json<Vec<ChromiumDatasetCompact>>, Error> {
    index_chromium_datasets(
        state,
        user,
        Json(ChromiumDatasetQuery::from_simple_query(q)),
    )
    .await
}
