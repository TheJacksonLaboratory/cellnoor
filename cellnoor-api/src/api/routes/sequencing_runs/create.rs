use axum::{Json, extract::State};
use cellnoor_models::sequencing_run::{NewSequencingRun, SequencingRun};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::{
    db::{self, DbConnection},
    state::AppState,
};

pub(super) async fn create_sequencing_run(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Json(sequencing_run): Json<NewSequencingRun>,
) -> Result<Json<SequencingRun>, db::Error> {
    insert_sequencing_run(sequencing_run, &mut db_conn)
        .await
        .map(Json)
}

pub async fn insert_sequencing_run(
    sequencing_run: NewSequencingRun,
    db_conn: &mut DbConnection,
) -> Result<SequencingRun, db::Error> {
    use cellnoor_schema::sequencing_runs::dsl::*;

    Ok(diesel::insert_into(sequencing_runs)
        .values(sequencing_run)
        .returning(SequencingRun::as_returning())
        .get_result(db_conn)
        .await?)
}
