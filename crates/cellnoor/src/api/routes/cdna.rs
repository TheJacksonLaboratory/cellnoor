use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::cdna::{CdnaCompact, CdnaQuery, SimpleCdnaQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::cdna::{
        create::create_cdna, delete::delete_cdna, index_compact::index_cdna,
        index_detailed::index_cdna_detailed, measurements::create::create_cdna_measurement,
        show::show_cdna, update::update_cdna,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_cdna).get(index_cdna_simple))
        .api_route("/search", post(index_cdna))
        .api_route("/search/detailed", post(index_cdna_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get(show_cdna).put(update_cdna).delete(delete_cdna),
        )
        .api_route("/measurements", post(create_cdna_measurement))
}

async fn index_cdna_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleCdnaQuery>,
) -> Result<Json<Vec<CdnaCompact>>, Error> {
    index_cdna(state, user, Json(CdnaQuery::from_simple_query(q))).await
}
