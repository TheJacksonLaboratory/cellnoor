use axum::extract::State;
use cellnoor_models::sequencing_run::SequencingRunIdLibraries;
use cellnoor_schema::sequencing_submissions;
use diesel::prelude::*;
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    api::{
        extract::{ValidPathJson, auth::AuthenticatedUser},
        routes::{ApiResponse, inner_handler},
    },
    db,
    state::AppState,
};

pub async fn add_libraries_to_sequencing_run(
    state: State<AppState>,
    user: AuthenticatedUser,
    ValidPathJson(sequencing_run_id, library_ids): ValidPathJson<
        SequencingRunIdLibraries,
        Vec<Uuid>,
    >,
) -> ApiResponse<()> {
    Ok((
        StatusCode::OK,
        inner_handler(state, user, (sequencing_run_id, library_ids)).await?,
    ))
}

impl db::Operation<()> for (SequencingRunIdLibraries, Vec<Uuid>) {
    fn execute(self, db_conn: &mut diesel::PgConnection) -> Result<(), db::Error> {
        let (sequencing_run_id, library_ids) = self;

        let seq_run_lib_map: Vec<_> = library_ids
            .iter()
            .map(|l| {
                (
                    sequencing_submissions::sequencing_run_id.eq(sequencing_run_id),
                    sequencing_submissions::library_id.eq(l),
                )
            })
            .collect();

        diesel::insert_into(sequencing_submissions::table)
            .values(&seq_run_lib_map)
            .execute(db_conn)?;

        Ok(())
    }
}
