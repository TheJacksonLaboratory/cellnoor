use scamplers_models::cdna::measurement::CdnaMeasurementCreation;

use crate::validate::Validate;

impl Validate for CdnaMeasurementCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        self.data().validate(db_conn)
    }
}
