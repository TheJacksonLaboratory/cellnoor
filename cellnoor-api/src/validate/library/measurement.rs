use cellnoor_models::library::{
    LibraryId, LibraryIdMeasurements, measurement::LibraryMeasurementCreation,
};
use diesel::PgConnection;
use jiff::Timestamp;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl Validate for (LibraryIdMeasurements, LibraryMeasurementCreation) {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        let (LibraryIdMeasurements(library_id), measurement) = self;
        measurement.data().validate(db_conn)?;
        validate_library_created_before_measurement(
            *library_id,
            measurement.measured_at(),
            db_conn,
        )?;
        Ok(())
    }
}

fn validate_library_created_before_measurement(
    library_id: impl Into<LibraryId>,
    measured_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let library_preparation_time = library_id.into().execute(db_conn)?.prepared_at();
    validate_timestamps(library_preparation_time, measured_at, "measured_at")?;

    Ok(())
}
