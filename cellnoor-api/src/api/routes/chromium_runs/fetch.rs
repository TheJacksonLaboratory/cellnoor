use axum::{extract::State, http::StatusCode};
use cellnoor_models::chromium_run::{ChromiumRun, ChromiumRunId};
use cellnoor_schema::chromium_runs::dsl::*;
use diesel::prelude::*;

use crate::{
    api::{
        extract::auth::AuthenticatedUser,
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub(super) async fn fetch_chromium_run(
    request: ChromiumRunId,
    state: State<AppState>,
    user: AuthenticatedUser,
) -> ApiResponse<ChromiumRun> {
    let item = inner_handler(state, user, request).await?;
    Ok((StatusCode::OK, item))
}

impl db::Operation<ChromiumRun> for ChromiumRunId {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<ChromiumRun, db::Error> {
        Ok(ChromiumRun::query().filter(id.eq(self)).first(db_conn)?)
    }
}
