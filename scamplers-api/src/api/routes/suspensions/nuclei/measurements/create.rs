use diesel::SelectableHelper;
use scamplers_models::suspension::measurements::{
    NucleusSuspensionMeasurementCreation, SuspensionMeasurement,
};
use scamplers_schema::suspension_measurements::dsl::*;

use crate::db;

impl db::Operation<SuspensionMeasurement> for NucleusSuspensionMeasurementCreation {
    fn execute(
        self,
        db_conn: &mut diesel::PgConnection,
    ) -> Result<SuspensionMeasurement, db::Error> {
        Ok(diesel::insert_into(suspension_measurements)
            .values(self.0)
            .returning(SuspensionMeasurement::as_returning())
            .get_result(db_conn)?)
    }
}
