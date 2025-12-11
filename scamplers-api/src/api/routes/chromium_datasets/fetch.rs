use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use scamplers_models::chromium_dataset::{ChromiumDataset, ChromiumDatasetId};
use scamplers_schema::chromium_datasets;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_chromium_dataset(
    request: ChromiumDatasetId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<ChromiumDataset> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<ChromiumDataset> for ChromiumDatasetId {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<ChromiumDataset, db::Error> {
        Ok(ChromiumDataset::query()
            .filter(chromium_datasets::id.eq(self))
            .first(db_conn)?)
    }
}
