use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::{
    chromium_dataset::ChromiumDatasetSummary, specimen::SpecimenIdChromiumDatasets,
};
use cellnoor_schema::specimens;
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{
            ApiResponse, chromium_datasets::chromium_datasets_to_all_specimens, handle_api_request,
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
        handle_api_request(state, user, specimen_id).await?,
    ))
}

impl db::Operation<Vec<ChromiumDatasetSummary>> for SpecimenIdChromiumDatasets {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<Vec<ChromiumDatasetSummary>, db::Error> {
        Ok(chromium_datasets_to_all_specimens()
            .select(ChromiumDatasetSummary::as_select())
            .filter(specimens::id.eq(self))
            .load(db_conn)?)
    }
}
