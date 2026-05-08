use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use cellnoor_types::institution::{Institution, InstitutionQuery, SimpleInstitutionQuery};

use crate::{
    auth::AuthUser,
    error::Error,
    handlers::institutions::{
        create::create_institution, delete::delete_institution, index::index_institutions,
        show::show_institution, update::update_institution,
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
    ApiRouter::new().api_route(
        "/",
        get(show_institution)
            .put(update_institution)
            .delete(delete_institution),
    )
}

async fn index_institutions_simple(
    state: State<AppState>,
    user: AuthUser,
    Query(q): Query<SimpleInstitutionQuery>,
) -> Result<Json<Vec<Institution>>, Error> {
    index_institutions(state, user, Json(InstitutionQuery::from_simple_query(q))).await
}
