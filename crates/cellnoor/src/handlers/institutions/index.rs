use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, InstitutionQuery};

use crate::{auth::AuthUser, error::Error, state::AppState};

pub async fn index_institutions(
    State(state): State<AppState>,
    user: AuthUser,
    query: serde_qs::axum::QsQuery<InstitutionQuery>,
) -> Result<Json<Vec<Institution>>, Error> {
    todo!()
}
