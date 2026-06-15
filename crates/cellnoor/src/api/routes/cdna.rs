use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::cdna::{CdnaCompact, CdnaDetailed, CdnaQuery, SimpleCdnaQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::cdna::{
        create_cdna, create_cdna_measurement, delete_cdna, index_cdna, index_cdna_detailed,
        show_cdna, update_cdna,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_cdna).get(index_cdna_simple))
        .api_route("/chromium/detailed", get(index_cdna_detailed_simple))
        .api_route("/search", post(index_cdna))
        .api_route("/chromium/search/detailed", post(index_cdna_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_cdna).put(update_cdna).delete(delete_cdna))
        .api_route("/measurements", post(create_cdna_measurement))
}

async fn index_cdna_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleCdnaQuery>,
) -> Result<Json<Vec<CdnaCompact>>, Error> {
    index_cdna(state, user, Json(CdnaQuery::from_simple_query(q))).await
}

async fn index_cdna_detailed_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleCdnaQuery>,
) -> Result<Json<Vec<CdnaDetailed>>, Error> {
    index_cdna_detailed(state, user, Json(CdnaQuery::from_simple_query(q))).await
}
