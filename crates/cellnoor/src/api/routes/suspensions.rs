use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleQuery,
    suspension::{Suspension, SuspensionQuery, SuspensionSortField},
};
use serde_qs::web::QsQuery;

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::suspensions::{
        create::create_suspension, index::index_suspensions,
        measurements::create::create_suspension_measurement, show::show_suspension,
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
        .api_route("/", get(show_suspension))
        .api_route("/measurements", post(create_suspension_measurement))
}

async fn index_suspensions_simple(
    state: State<AppState>,
    user: AuthUser,
    QsQuery(q): QsQuery<SimpleQuery<SuspensionSortField>>,
) -> Result<Json<Vec<Suspension>>, Error> {
    index_suspensions(state, user, Json(SuspensionQuery::from_simple_query(q))).await
}
