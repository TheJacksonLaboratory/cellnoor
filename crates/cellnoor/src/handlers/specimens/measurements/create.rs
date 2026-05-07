use axum::{
    Json,
    extract::{Path, State},
};
use cellnoor_types::{
    IdParam,
    specimen::measurement::{NewSpecimenMeasurement, SpecimenMeasurement, SpecimenMeasurementData},
};
use postgres_types::Json as PgJson;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    db,
    error::{Error, ErrorInner},
    state::AppState,
};

pub async fn insert_specimen_measurement(
    tx: &db::Transaction<'_>,
    specimen_id: Uuid,
    NewSpecimenMeasurement {
        measured_by,
        measured_at,
        data,
    }: &NewSpecimenMeasurement,
) -> Result<(), Error> {
    validate_specimen_measurement_data(data)?;

    let data_json = PgJson(data);
    tx.execute(
        "insert into specimen_measurement (specimen_id, measured_by, measured_at, data) values \
         ($1, $2, $3, $4) returning id",
        &[&specimen_id, measured_by, measured_at, &data_json],
    )
    .await?;

    Ok(())
}

pub(super) fn validate_specimen_measurement_data(
    data: &SpecimenMeasurementData,
) -> Result<(), Error> {
    let (quantity, value, min, max) = match *data {
        SpecimenMeasurementData::Dv200 { value, .. } => ("DV200", f32::from(value), 0.0, 1.0),
        SpecimenMeasurementData::Rin { value, .. } => ("RIN", f32::from(value), 1.0, 10.0),
    };

    if value < min || value > max {
        return Err(Error {
            error: ErrorInner::DataConstraint {
                resource: Some("specimen_measurement".to_owned()),
                field: Some("data.value".to_owned()),
                message: format!("{quantity} value must be between {min} and {max}"),
                detail: None,
            },
        });
    }

    Ok(())
}
