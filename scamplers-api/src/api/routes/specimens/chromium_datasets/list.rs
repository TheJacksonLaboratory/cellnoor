use axum::{extract::State, http::StatusCode};
use diesel::prelude::*;
use scamplers_models::{
    chromium_dataset::ChromiumDatasetSummary, specimen::SpecimenIdChromiumDatasets,
};
use scamplers_schema::{chromium_datasets, specimens};
use uuid::Uuid;

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

pub async fn list_chromium_datasets(
    specimen_id: SpecimenIdChromiumDatasets,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<ChromiumDatasetSummary>> {
    Ok((
        StatusCode::OK,
        inner_handler(state, user, specimen_id).await?,
    ))
}

impl db::Operation<Vec<ChromiumDatasetSummary>> for SpecimenIdChromiumDatasets {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<ChromiumDatasetSummary>, db::Error> {
        let filter = specimens::id.eq(&self);

        let dataset_ids_derived_from_suspension_pool: Vec<Uuid> =
            chromium_datasets_to_pooled_specimens()
                .select(chromium_datasets::id)
                .filter(filter)
                .load(db_conn)?;

        let dataset_ids_derived_from_single_suspension = chromium_datasets_to_specimens()
            .select(chromium_datasets::id)
            .filter(filter)
            .load(db_conn)?;

        let dataset_ids = dataset_ids_derived_from_suspension_pool
            .into_iter()
            .chain(dataset_ids_derived_from_single_suspension);

        Ok(chromium_datasets::table
            .filter(chromium_datasets::id.eq_any(dataset_ids))
            .select(ChromiumDatasetSummary::as_select())
            .order_by(chromium_datasets::delivered_at)
            .load(db_conn)?)
    }
}
