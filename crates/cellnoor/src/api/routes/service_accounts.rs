use aide::axum::{
    ApiRouter,
    routing::{post, put},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::service_account::{
    ServiceAccount, ServiceAccountQuery, SimpleServiceAccountQuery,
};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::service_accounts::{
        add_people_to_service_account, create_service_account, delete_service_account,
        index_service_accounts, update_service_account,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            post(create_service_account).get(index_service_accounts_simple),
        )
        .api_route("/search", post(index_service_accounts))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            put(update_service_account).delete(delete_service_account),
        )
        .api_route("/people", post(add_people_to_service_account))
}

async fn index_service_accounts_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleServiceAccountQuery>,
) -> Result<Json<Vec<ServiceAccount>>, Error> {
    index_service_accounts(state, user, Json(ServiceAccountQuery::from_simple_query(q))).await
}
