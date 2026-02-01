use axum::Json;
use axum::extract::{Path, State};
use cellnoor_models::IdParameter;
use cellnoor_models::specimen::measurement::{NewSpecimenMeasurement, SpecimenMeasurement};
use cellnoor_schema::{specimen_measurements, specimens};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

use crate::api::util::validate_timestamps;
use crate::db::{self, DbConnection};
use crate::state::AppState;

pub async fn create_specimen_measurement(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewSpecimenMeasurement>,
) -> Result<Json<SpecimenMeasurement>, db::Error> {
    let specimen_received_at = fetch_specimen_time_of_receipt(id, &mut db_conn).await?;

    validate_timestamps(
        (specimen_received_at, "specimen_received_at"),
        (measurement.measured_at(), "measurement_made_at"),
    )?;

    insert_specimen_measurement(id, measurement, &mut db_conn)
        .await
        .map(Json)
}

async fn fetch_specimen_time_of_receipt(
    specimen_id: Uuid,
    db_conn: &mut DbConnection,
) -> Result<Timestamp, db::Error> {
    Ok(specimens::table
        .select(specimens::received_at)
        .filter(specimens::id.eq(specimen_id))
        .first(db_conn)
        .await
        .map(jiff_diesel::Timestamp::to_jiff)?)
}

async fn insert_specimen_measurement(
    specimen_id: Uuid,
    measurement: NewSpecimenMeasurement,
    db_conn: &mut DbConnection,
) -> Result<SpecimenMeasurement, db::Error> {
    Ok(diesel::insert_into(specimen_measurements::table)
        .values((
            specimen_measurements::specimen_id.eq(specimen_id),
            measurement,
        ))
        .returning(SpecimenMeasurement::as_returning())
        .get_result(db_conn)
        .await?)
}
