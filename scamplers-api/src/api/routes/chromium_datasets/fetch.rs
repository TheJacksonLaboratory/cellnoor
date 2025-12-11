use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use scamplers_models::chromium_dataset::{ChromiumDataset, ChromiumDatasetId};
use scamplers_schema::chromium_datasets;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{
            ApiResponse,
            chromium_datasets::common::{
                chromium_datasets_to_pooled_specimens, chromium_datasets_to_specimens,
            },
            inner_handler,
        },
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
        let filter = chromium_datasets::id.eq(&self);

        let pooled = chromium_datasets_to_pooled_specimens()
            .select(ChromiumDataset::as_select())
            .filter(filter)
            .first(db_conn)
            .optional()?;

        if let Some(ds) = pooled {
            return Ok(ds);
        }

        // If we couldn't find a dataset that derives from a suspension pool, then we
        // know it derived from individual suspensions
        Ok(chromium_datasets_to_specimens()
            .select(ChromiumDataset::as_select())
            .filter(filter)
            .first(db_conn)?)
    }
}
