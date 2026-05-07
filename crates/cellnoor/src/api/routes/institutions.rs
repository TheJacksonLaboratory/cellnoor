use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{Json, extract::State};
use cellnoor_types::{
    ComplexQuery, SimpleQuery,
    institution::{Institution, InstitutionSortField},
};
use serde_qs::web::QsQuery;

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::institutions::{
        create::create_institution, index::index_institutions, show::show_institution,
    },
    state::AppState,
};

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post(create_institution).get(index_institutions_simple))
        .api_route("/search", post(index_institutions))
        .nest("/{id}", id_router())
}

fn id_router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route("/", get(show_institution))
}

async fn index_institutions_simple(
    state: State<AppState>,
    user: AuthUser,
    QsQuery(q): QsQuery<SimpleQuery<InstitutionSortField>>,
) -> Result<Json<Vec<Institution>>, Error> {
    index_institutions(state, user, Json(ComplexQuery::from_simple_query(q))).await
}
