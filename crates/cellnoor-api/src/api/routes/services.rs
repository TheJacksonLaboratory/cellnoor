use aide::axum::{
    ApiRouter,
    routing::{post, put},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::service::{Service, ServiceQuery, ServiceSimpleFields, SimpleServiceQuery};

use crate::{
    auth::AuthUser,
    db::DbError,
    handlers::{
        delete_resource,
        services::{add_people_to_service, create_service, index_services, update_service},
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
    // Deleting a service fires a trigger that drops the principal, which
    // cascades to who has access to the service and to its API keys
    ApiRouter::new()
        .api_route(
            "/",
            put(update_service).delete(delete_resource::<ServiceSimpleFields>),
        )
        .api_route("/people", post(add_people_to_service))
}

async fn index_services_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleServiceQuery>,
) -> Result<Json<Vec<Service>>, DbError> {
    index_services(
        state,
        user,
        crate::extract::JsonExtractor(ServiceQuery::from_simple_query(q)),
    )
    .await
}
