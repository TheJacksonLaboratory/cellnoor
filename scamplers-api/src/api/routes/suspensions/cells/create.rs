use axum::extract::State;
use reqwest::StatusCode;
use scamplers_models::suspension::{Suspension, SuspensionContent, SuspensionCreation};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    state::AppState,
};

pub(super) async fn create_cell_suspension(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SuspensionCreation>,
) -> ApiResponse<Suspension> {
    let item = inner_handler(state, user, (request, SuspensionContent::Cells)).await?;
    Ok((StatusCode::CREATED, item))
}
