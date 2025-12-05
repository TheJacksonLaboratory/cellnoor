use diesel::PgConnection;
use jiff::Timestamp;
use scamplers_models::library::{LibraryId, measurement::LibraryMeasurementCreation};

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

impl Validate for LibraryMeasurementCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        self.data().validate(db_conn)?;
        validate_measurement_time(self.library_id(), self.measured_at(), db_conn)?;
        Ok(())
    }
}

fn validate_measurement_time(
    library_id: impl Into<LibraryId>,
    measured_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), crate::validate::Error> {
    let library_preparation_time = library_id.into().execute(db_conn)?.prepared_at();
    validate_timestamps(library_preparation_time, measured_at, "measured_at")?;

    Ok(())
}
