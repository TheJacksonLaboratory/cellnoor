use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::suspension::{SimpleSuspensionQuery, SuspensionCompact, SuspensionQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::suspensions::{
        create_suspension, create_suspension_measurement, delete_suspension, index_suspensions,
        index_suspensions_detailed, show_suspension, update_suspension,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_suspension).get(index_suspensions_simple))
        .api_route("/search", post(index_suspensions))
        .api_route("/search/detailed", post(index_suspensions_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get(show_suspension)
                .put(update_suspension)
                .delete(delete_suspension),
        )
        .api_route("/measurements", post(create_suspension_measurement))
}

async fn index_suspensions_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleSuspensionQuery>,
) -> Result<Json<Vec<SuspensionCompact>>, Error> {
    index_suspensions(state, user, Json(SuspensionQuery::from_simple_query(q))).await
}
