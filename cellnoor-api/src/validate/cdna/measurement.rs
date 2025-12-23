use cellnoor_models::cdna::{CdnaId, CdnaIdMeasurements, measurement::CdnaMeasurementCreation};
use diesel::PgConnection;
use jiff::Timestamp;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl Validate for (CdnaIdMeasurements, CdnaMeasurementCreation) {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        let (CdnaIdMeasurements(cdna_id), measurement) = self;
        measurement.data().validate(db_conn)?;
        validate_cdna_created_before_measurement(*cdna_id, measurement.measured_at(), db_conn)?;

        Ok(())
    }
}

fn validate_cdna_created_before_measurement(
    cdna_id: impl Into<CdnaId>,
    measured_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let cdna_preparation_time = cdna_id.into().execute(db_conn)?.prepared_at();
    validate_timestamps(cdna_preparation_time, measured_at, "measured_at")?;

    Ok(())
}
