use aide::OperationIo;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, response::IntoResponse};
use cellnoor_models::IdParameter;
use cellnoor_models::specimen::measurement::{NewSpecimenMeasurement, SpecimenMeasurement};
use cellnoor_schema::{specimen_measurements, specimens};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use jiff::Timestamp;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::db::{self, DbConnection};
use crate::state::AppState;

pub async fn create_specimen_measurement(
    _: State<AppState>,
    mut db_conn: DbConnection,
    Path(IdParameter { id }): Path<IdParameter>,
    Json(measurement): Json<NewSpecimenMeasurement>,
) -> Result<Json<SpecimenMeasurement>, Error> {
    let specimen_received_at = fetch_specimen_time_of_receipt(id, &mut db_conn).await?;

    validate_specimen_received_before_measurement(specimen_received_at, measurement.measured_at())?;

    Ok(insert_specimen_measurement(id, measurement, &mut db_conn)
        .await
        .map(Json)?)
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

fn validate_specimen_received_before_measurement(
    specimen_received_at: Timestamp,
    measurement_made_at: Timestamp,
) -> Result<(), Error> {
    if specimen_received_at > measurement_made_at {
        return Err(Error::MeasurementMadeBeforeSpecimenReceived);
    }

    Ok(())
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

#[derive(Debug, thiserror::Error, Serialize, JsonSchema, OperationIo)]
#[serde(rename_all = "snake_case", tag = "type")]
#[schemars(rename = "CreateSpecimenMeasurementError")]
#[error(transparent)]
pub enum Error {
    Database(#[from] db::Error),
    #[error("measurement made before specimen received")]
    MeasurementMadeBeforeSpecimenReceived,
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Self::Database(err.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Database(e) => e.into_response(),
            Self::MeasurementMadeBeforeSpecimenReceived => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response()
            }
        }
    }
}
