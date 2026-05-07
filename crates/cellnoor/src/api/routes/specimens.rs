use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{Json, extract::State};
use cellnoor_types::{
    SimpleQuery,
    specimen::{Specimen, SpecimenQuery, SpecimenSortField},
};
use serde_qs::web::QsQuery;

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::specimens::{
        create::create_specimen, index::index_specimens,
        measurements::create::create_specimen_measurement, show::show_specimen,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_specimen).get(index_specimens_simple))
        .api_route("/search", post(index_specimens))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(show_specimen))
        .api_route("/measurements", post(create_specimen_measurement))
}

async fn index_specimens_simple(
    state: State<AppState>,
    user: AuthUser,
    QsQuery(q): QsQuery<SimpleQuery<SpecimenSortField>>,
) -> Result<Json<Vec<Specimen>>, Error> {
    index_specimens(state, user, Json(SpecimenQuery::from_simple_query(q))).await
}
