use axum::extract::State;
use reqwest::StatusCode;
use scamplers_models::{
    suspension::SuspensionContent,
    suspension_pool::{SuspensionPool, SuspensionPoolCreation},
};

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    state::AppState,
};

pub(super) async fn create_cell_suspension_pool(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SuspensionPoolCreation>,
) -> ApiResponse<SuspensionPool> {
    let item = inner_handler(state, user, (request, SuspensionContent::Cells)).await?;
    Ok((StatusCode::CREATED, item))
}
