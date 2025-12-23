use cellnoor_models::specimen::{
    SpecimenId, SpecimenIdMeasurements, measurement::SpecimenMeasurementCreation,
};
use diesel::PgConnection;
use jiff::Timestamp;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl Validate for (SpecimenIdMeasurements, SpecimenMeasurementCreation) {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        let (SpecimenIdMeasurements(specimen_id), measurement) = self;
        validate_specimen_received_before_measurement(
            *specimen_id,
            measurement.measured_at(),
            db_conn,
        )?;

        Ok(())
    }
}

fn validate_specimen_received_before_measurement(
    specimen_id: impl Into<SpecimenId>,
    measured_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let specimen_received_at = specimen_id.into().execute(db_conn)?.received_at();
    validate_timestamps(specimen_received_at, measured_at, "measured_at")?;

    Ok(())
}
