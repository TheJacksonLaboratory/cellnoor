use aide::axum::{
    ApiRouter,
    routing::{post, put},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::service::{Service, ServiceQuery, SimpleServiceQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::services::{
        add_people_to_service, create_service, delete_service, index_services, update_service,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_service).get(index_services_simple))
        .api_route("/search", post(index_services))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", put(update_service).delete(delete_service))
        .api_route("/people", post(add_people_to_service))
}

async fn index_services_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleServiceQuery>,
) -> Result<Json<Vec<Service>>, Error> {
    index_services(state, user, Json(ServiceQuery::from_simple_query(q))).await
}
