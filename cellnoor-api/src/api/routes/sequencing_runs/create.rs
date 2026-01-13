use axum::{extract::State, http::status::StatusCode};
use cellnoor_models::sequencing_run::{SequencingRun, SequencingRunCreation};
use diesel::prelude::*;

use crate::{
    api::{
        extract::{ValidJson, auth::AuthenticatedUser},
        routes::{ApiResponse, Root, handle_request},
    },
    db,
    state::AppState,
};

pub(super) async fn create_sequencing_run(
    _: Root,
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidJson(request): ValidJson<SequencingRunCreation>,
) -> ApiResponse<SequencingRun> {
    let item = handle_request(state, user, request).await?;
    Ok((StatusCode::CREATED, item))
}

impl db::Operation<SequencingRun> for SequencingRunCreation {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<SequencingRun, db::Error> {
        use cellnoor_schema::sequencing_runs::dsl::*;

        Ok(diesel::insert_into(sequencing_runs)
            .values(self)
            .returning(SequencingRun::as_returning())
            .get_result(db_conn)?)
    }
}
