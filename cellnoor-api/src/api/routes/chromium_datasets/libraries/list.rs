use axum::{extract::State, http::StatusCode};
use cellnoor_models::{chromium_dataset::ChromiumDatasetIdLibraries, library::LibrarySummary};
use cellnoor_schema::{chromium_dataset_libraries, chromium_datasets, libraries};
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn list_libraries(
    dataset_id: ChromiumDatasetIdLibraries,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<Vec<LibrarySummary>> {
    Ok((
        StatusCode::OK,
        inner_handler(state, user, dataset_id).await?,
    ))
}

impl db::Operation<Vec<LibrarySummary>> for ChromiumDatasetIdLibraries {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<Vec<LibrarySummary>, db::Error> {
        Ok(chromium_datasets::table
            .inner_join(chromium_dataset_libraries::table.inner_join(libraries::table))
            .select(LibrarySummary::as_select())
            .filter(chromium_datasets::id.eq(self))
            .load(db_conn)?)
    }
}
