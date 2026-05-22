use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::specimen::{SimpleSpecimenQuery, SpecimenCompact, SpecimenQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::specimens::{
        create_specimen, create_specimen_measurement, delete_specimen, index_specimens,
        index_specimens_detailed, show_specimen, update_specimen,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_specimen).get(index_specimens_simple))
        .api_route("/search", post(index_specimens))
        .api_route("/search/detailed", post(index_specimens_detailed))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route(
            "/",
            get(show_specimen)
                .put(update_specimen)
                .delete(delete_specimen),
        )
        .api_route("/measurements", post(create_specimen_measurement))
}

async fn index_specimens_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleSpecimenQuery>,
) -> Result<Json<Vec<SpecimenCompact>>, Error> {
    index_specimens(state, user, Json(SpecimenQuery::from_simple_query(q))).await
}
