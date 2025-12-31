use axum::extract::State;
use cellnoor_models::suspension::{Suspension, SuspensionContent, SuspensionCreation};
use reqwest::StatusCode;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    state::AppState,
};

pub(super) async fn create_nucleus_suspension(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SuspensionCreation>,
) -> ApiResponse<Suspension> {
    let item = inner_handler(state, user, (request, SuspensionContent::Nuclei)).await?;
    Ok((StatusCode::CREATED, item))
}
