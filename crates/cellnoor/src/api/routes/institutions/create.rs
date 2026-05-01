use axum::{Json, extract::State};
use cellnoor_types::institution::{Institution, NewInstitution};

use crate::{auth::AuthUser, db::institution::insert_institution, error::Error, state::AppState};

pub async fn create_institution(
    State(state): State<AppState>,
    user: AuthUser,
    Json(institution): Json<NewInstitution>,
) -> Result<Json<Institution>, Error> {
    insert_institution(institution, &mut state.db_client(user).await?)
        .await
        .map(Json)
}
