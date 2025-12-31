use axum::{extract::State, http::StatusCode, response::Html};
use cellnoor_models::chromium_dataset::ChromiumDatasetWebSummaryFilename;
use cellnoor_schema::chromium_dataset_web_summaries;
use diesel::prelude::*;

use crate::{
    api::{self, extract::auth::AuthenticatedUser},
    db::{self, Operation},
    state::AppState,
};
#[axum::debug_handler]
pub async fn fetch_web_summary(
    web_summary_path: ChromiumDatasetWebSummaryFilename,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> Result<(StatusCode, Html<Vec<u8>>), api::ErrorResponse> {
    tracing::info!(
        "fetching web summary {} for Chromium dataset {}",
        web_summary_path.1,
        web_summary_path.0
    );

    let db_conn = state.db_conn().await?;
    let file = db_conn
        .interact(move |db_conn| web_summary_path.execute_as_user(user.id(), db_conn))
        .await??;

    Ok((StatusCode::OK, Html(file)))
}

impl db::Operation<Vec<u8>> for ChromiumDatasetWebSummaryFilename {
    fn execute(self, db_conn: &mut PgConnection) -> Result<Vec<u8>, db::Error> {
        let Self(dataset_id, directory, filename) = self;

        Ok(chromium_dataset_web_summaries::table
            .select(chromium_dataset_web_summaries::content)
            .filter(chromium_dataset_web_summaries::dataset_id.eq(dataset_id))
            .filter(chromium_dataset_web_summaries::directory.eq(directory))
            .filter(chromium_dataset_web_summaries::filename.eq(filename))
            .first(db_conn)?)
    }
}
