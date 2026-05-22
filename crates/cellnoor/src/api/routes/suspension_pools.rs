use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::suspension_pool::{
    SimpleSuspensionPoolQuery, SuspensionPoolCompact, SuspensionPoolQuery,
};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::suspension_pools::{
        create_suspension_pool, create_suspension_pool_measurement, delete_suspension_pool,
        index_suspension_pools, index_suspension_pools_detailed, show_suspension_pool,
        update_suspension_pool,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_suspension_pool).get(index_suspension_pools_simple),
        )
        .api_route("/search", post(index_suspension_pools))
        .api_route("/search/detailed", post(index_suspension_pools_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get(show_suspension_pool)
                .put(update_suspension_pool)
                .delete(delete_suspension_pool),
        )
        .api_route("/measurements", post(create_suspension_pool_measurement))
}

async fn index_suspension_pools_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleSuspensionPoolQuery>,
) -> Result<Json<Vec<SuspensionPoolCompact>>, Error> {
    index_suspension_pools(state, user, Json(SuspensionPoolQuery::from_simple_query(q))).await
}
