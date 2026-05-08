use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::suspension::{SimpleSuspensionQuery, Suspension, SuspensionQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::suspensions::{
        create::create_suspension, delete::delete_suspension, index::index_suspensions,
        measurements::create::create_suspension_measurement, show::show_suspension,
        update::update_suspension,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_suspension).get(index_suspensions_simple))
        .api_route("/search", post(index_suspensions))
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
) -> Result<Json<Vec<Suspension>>, Error> {
    index_suspensions(state, user, Json(SuspensionQuery::from_simple_query(q))).await
}
