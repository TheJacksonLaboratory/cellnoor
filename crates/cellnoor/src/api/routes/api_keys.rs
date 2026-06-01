use aide::axum::{
    ApiRouter,
    routing::{post, put},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::api_key::{ApiKeyQuery, ApiKeyRecord, SimpleApiKeyQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::api_keys::{create_api_key, delete_api_key, index_api_keys, update_api_key},
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_api_key).get(index_api_keys_simple))
        .api_route("/search", post(index_api_keys))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", put(update_api_key).delete(delete_api_key))
}

async fn index_api_keys_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleApiKeyQuery>,
) -> Result<Json<Vec<ApiKeyRecord>>, Error> {
    index_api_keys(state, user, Json(ApiKeyQuery::from_simple_query(q))).await
}
