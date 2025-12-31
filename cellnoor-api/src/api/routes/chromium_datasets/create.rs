use axum::{extract::State, http::StatusCode};
use cellnoor_models::chromium_dataset::{
    ChromiumDataset, ChromiumDatasetCreation, ChromiumDatasetId,
};
use cellnoor_schema::{chromium_dataset_libraries, chromium_datasets};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn create_chromium_dataset(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<ChromiumDatasetCreation>,
) -> ApiResponse<ChromiumDataset> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<ChromiumDataset> for ChromiumDatasetCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<ChromiumDataset, db::Error> {
        let library_ids = self.library_ids().to_owned();

        let dataset_id: ChromiumDatasetId = diesel::insert_into(chromium_datasets::table)
            .values(self)
            .returning(chromium_datasets::id)
            .get_result(db_conn)?;

        insert_chromium_dataset_libraries(dataset_id, &library_ids, db_conn)?;

        dataset_id.execute(db_conn)
    }
}

fn insert_chromium_dataset_libraries(
    dataset_id: ChromiumDatasetId,
    library_ids: &[Uuid],
    db_conn: &mut diesel::PgConnection,
) -> Result<(), db::Error> {
    let ds_lib_map: Vec<_> = library_ids
        .iter()
        .map(|l| {
            (
                chromium_dataset_libraries::dataset_id.eq(dataset_id),
                chromium_dataset_libraries::library_id.eq(l),
            )
        })
        .collect();

    diesel::insert_into(chromium_dataset_libraries::table)
        .values(ds_lib_map)
        .execute(db_conn)?;

    Ok(())
}
