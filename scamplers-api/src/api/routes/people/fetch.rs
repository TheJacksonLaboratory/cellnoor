use axum::{extract::State, http::StatusCode};
use scamplers_models::person::{Person, PersonId};

use super::{ApiResponse, inner_handler};
use crate::{api::extract::auth::AuthenticatedUser, state::AppState};

pub(super) async fn fetch_person(
    request: PersonId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Person> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::FOUND, item))
}
