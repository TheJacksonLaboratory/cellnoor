use axum::{Extension, Json, extract::State};
use cellnoor_models::specimen::{NewSpecimen, Specimen};
use cellnoor_schema::{projects, specimens};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::{auth::AuthUser, util::validate_timestamps},
    db::{self, DbConnection, jiff_diesel_tuple_to_jiff},
    state::AppState,
};

pub async fn create_specimen(
    _: State<AppState>,
    db_conn: DbConnection,
    Extension(user): Extension<AuthUser>,
    Json(specimen): Json<NewSpecimen>,
) -> Result<Json<Specimen>, db::Error> {
    let (project_start_date, project_end_date) =
        project_start_and_end_date(specimen.project_id(), &db_conn).await?;

    // Should I spawn two threads for these tasks? Since I love hyper-optimizing
    validate_timestamps(
        (project_start_date, "project_start_date"),
        (specimen.received_at(), "specimen_received_at"),
    )?;

    validate_timestamps(
        (specimen.received_at(), "specimen_received_at"),
        (project_end_date, "project_end_date"),
    )?;

    let id = insert_specimen(specimen, &db_conn).await?;

    super::show::select_specimen_by_id(user.projects(), id, &db_conn)
        .await
        .map(Json)
}

pub async fn insert_specimen(
    specimen: NewSpecimen,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Uuid, db::Error> {
    let split = match specimen {
        NewSpecimen::Block(s) => s.split_for_insertion(),
        NewSpecimen::CellPellet(s) => s.split_for_insertion(),
        NewSpecimen::Suspension(s) => s.split_for_insertion(),
        NewSpecimen::Tissue(s) => s.split_for_insertion(),
    };

    Ok(diesel::insert_into(specimens::table)
        .values(split)
        .returning(specimens::id)
        .get_result(&mut db_conn)
        .await?)
}

async fn project_start_and_end_date(
    project_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<(Timestamp, Timestamp), db::Error> {
    Ok(projects::table
        .select((projects::started_at, projects::ended_at))
        .find(project_id)
        .first(&mut db_conn)
        .await
        .map(jiff_diesel_tuple_to_jiff)?)
}
