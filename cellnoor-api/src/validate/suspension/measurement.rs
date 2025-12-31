use cellnoor_models::suspension::{
    SuspensionId, SuspensionIdMeasurements, measurement::SuspensionMeasurementFields,
};
use diesel::PgConnection;
use jiff::Timestamp;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl<C> Validate for (SuspensionIdMeasurements, SuspensionMeasurementFields<C>) {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        let (SuspensionIdMeasurements(suspension_id), measurement) = self;
        validate_suspension_created_or_received_before_measurement(
            *suspension_id,
            measurement.measured_at(),
            db_conn,
        )?;

        Ok(())
    }
}

fn validate_suspension_created_or_received_before_measurement(
    suspension_id: impl Into<SuspensionId>,
    measured_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let suspension = suspension_id.into().execute(db_conn)?;
    let first_timestamp = match suspension.created_at() {
        Some(t) => t,
        None => suspension.parent_specimen_received_at(),
    };

    validate_timestamps(first_timestamp, measured_at, "measured_at")?;

    Ok(())
}
