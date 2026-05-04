use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_models::{
    IdParameter,
    specimen::measurement::{NewSpecimenMeasurement, SpecimenMeasurement},
};
use cellnoor_schema::{specimen_measurements, specimens};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    api::util::validate_timestamps,
    db::{self, DbConnection},
    state::AppState,
};

pub async fn create_specimen_measurement(
    _: State<AppState>,
    db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewSpecimenMeasurement>,
) -> Result<Json<SpecimenMeasurement>, db::Error> {
    let specimen_received_at = specimen_received_at(id, &db_conn).await?;

    validate_timestamps(
        (specimen_received_at, "specimen_received_at"),
        (measurement.measured_at(), "measurement_made_at"),
    )?;

    insert_specimen_measurement(id, measurement, &db_conn)
        .await
        .map(Json)
}

async fn specimen_received_at(
    specimen_id: Uuid,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<Timestamp, db::Error> {
    Ok(specimens::table
        .select(specimens::received_at)
        .filter(specimens::id.eq(specimen_id))
        .first(&mut db_conn)
        .await
        .map(jiff_diesel::Timestamp::to_jiff)?)
}

async fn insert_specimen_measurement(
    specimen_id: Uuid,
    measurement: NewSpecimenMeasurement,
    mut db_conn: &diesel_async::AsyncPgConnection,
) -> Result<SpecimenMeasurement, db::Error> {
    Ok(diesel::insert_into(specimen_measurements::table)
        .values((
            specimen_measurements::specimen_id.eq(specimen_id),
            measurement,
        ))
        .returning(SpecimenMeasurement::as_returning())
        .get_result(&mut db_conn)
        .await?)
}
