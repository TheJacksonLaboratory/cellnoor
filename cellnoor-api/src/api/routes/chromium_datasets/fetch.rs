use axum::{extract::State, http::StatusCode};
use cellnoor_models::chromium_dataset::{ChromiumDataset, ChromiumDatasetId};
use cellnoor_schema::{
    cdna, chromium_dataset_libraries, chromium_datasets, chromium_runs, gem_pools, labs, libraries,
    tenx_assays,
};
use diesel::prelude::*;

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
        Ok(chromium_datasets_to_assay()
            .select(ChromiumDataset::as_select())
            .filter(chromium_datasets::id.eq(self))
            .first(db_conn)?)
    }
}

#[diesel::dsl::auto_type]
fn chromium_datasets_to_assay() -> _ {
    chromium_datasets::table.inner_join(labs::table).inner_join(
        chromium_dataset_libraries::table.inner_join(libraries::table.inner_join(
            cdna::table.inner_join(
                gem_pools::table.inner_join(chromium_runs::table.inner_join(tenx_assays::table)),
            ),
        )),
    )
}
