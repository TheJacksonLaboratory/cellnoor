use cellnoor_models::suspension::{
    Suspension, SuspensionContent, SuspensionId, SuspensionIdMeasurements,
    measurement::{
        CellSuspensionMeasurementCreation, NucleusSuspensionMeasurementCreation,
        SuspensionMeasurementFields,
    },
};
use diesel::PgConnection;
use jiff::Timestamp;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "typescript",
    ts(rename = "SuspensionMeasurementValidationError")
)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Error {
    #[error("expected {expected_suspension_content}-suspension measurement")]
    WrongSuspensionContent {
        expected_suspension_content: SuspensionContent,
    },
}

impl Validate for (SuspensionIdMeasurements, CellSuspensionMeasurementCreation) {
    fn validate(&self, db_conn: &mut PgConnection) -> Result<(), crate::validate::Error> {
        validate_suspension_measurement(self, db_conn)?;

        Ok(())
    }
}

impl Validate
    for (
        SuspensionIdMeasurements,
        NucleusSuspensionMeasurementCreation,
    )
{
    fn validate(&self, db_conn: &mut PgConnection) -> Result<(), crate::validate::Error> {
        validate_suspension_measurement(self, db_conn)?;

        Ok(())
    }
}

fn validate_suspension_measurement<C>(
    (SuspensionIdMeasurements(suspension_id), measurement): &(
        SuspensionIdMeasurements,
        SuspensionMeasurementFields<C>,
    ),

    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error>
where
    SuspensionContent: TryInto<C>,
{
    let suspension_id: SuspensionId = (*suspension_id).into();
    let suspension = suspension_id.execute(db_conn)?;

    validate_suspension_measurement_content_matches_suspension_content(suspension.content())?;

    validate_suspension_created_or_received_before_measurement(
        &suspension,
        measurement.measured_at(),
    )?;

    Ok(())
}

fn validate_suspension_measurement_content_matches_suspension_content<C>(
    suspension_content: SuspensionContent,
) -> Result<(), crate::validate::Error>
where
    SuspensionContent: TryInto<C>,
{
    suspension_content
        .try_into()
        .map_err(|_| Error::WrongSuspensionContent {
            expected_suspension_content: suspension_content,
        })?;

    Ok(())
}

fn validate_suspension_created_or_received_before_measurement(
    suspension: &Suspension,
    measured_at: Timestamp,
) -> Result<(), crate::validate::Error> {
    let first_timestamp = suspension
        .created_at()
        .unwrap_or(suspension.parent_specimen_received_at());

    validate_timestamps(first_timestamp, measured_at, "measured_at")?;

    Ok(())
}
