use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::IdParameter;
use cellnoor_schema::{libraries, sequencing_runs, sequencing_submissions};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::util::validate_timestamps,
    db::{self, DbConnection, jiff_diesel_optional_tuple_to_jiff},
    state::AppState,
};

pub async fn add_libraries_to_sequencing_run(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(library_ids): Json<Vec<Uuid>>,
) -> Result<(), db::Error> {
    let ((run_begun_at, run_finished_at), libraries_prepared_at) = tokio::try_join!(
        sequencing_run_begun_at_and_finished_at(id, &db_conn),
        libraries_prepared_at(&library_ids, &db_conn)
    )?;

    for lib_prep_time in libraries_prepared_at {
        let lib_prep_time = (lib_prep_time, "library_prepared_at");

        validate_timestamps((run_begun_at, "sequencing_run_begun_at"), lib_prep_time)?;

        if let Some(run_finished_at) = run_finished_at {
            validate_timestamps(
                lib_prep_time,
                (run_finished_at, "sequencing_run_finished_at"),
            )?;
        }
    }

    insert_sequencing_run_library_mappings(id, &library_ids, &mut db_conn).await
}

pub async fn insert_sequencing_run_library_mappings(
    sequencing_run_id: Uuid,
    library_ids: &[Uuid],
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(), db::Error> {
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
        .execute(&mut db_conn)
        .await?;

    Ok(())
}

async fn sequencing_run_begun_at_and_finished_at(
    sequencing_run_id: Uuid,
    mut db_conn: &AsyncPgConnection,
) -> Result<(Timestamp, Option<Timestamp>), db::Error> {
    Ok(sequencing_runs::table
        .select((sequencing_runs::begun_at, sequencing_runs::finished_at))
        .find(sequencing_run_id)
        .first(&mut db_conn)
        .await
        .map(jiff_diesel_optional_tuple_to_jiff)?)
}

async fn libraries_prepared_at(
    library_ids: &[Uuid],
    mut db_conn: &AsyncPgConnection,
) -> Result<Vec<Timestamp>, db::Error> {
    Ok(libraries::table
        .select(libraries::prepared_at)
        .filter(libraries::id.eq_any(library_ids))
        .load(&mut db_conn)
        .await
        .map(|l| l.into_iter().map(jiff_diesel::Timestamp::to_jiff).collect())?)
}
