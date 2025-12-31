use cellnoor_models::tenx_assay::TenxAssayCreation;

use crate::validate::{Validate, initial_data::validate_10x_genomics_url};

impl Validate for TenxAssayCreation {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        validate_10x_genomics_url(&self.protocol_url())?;

        Ok(())
    }
}
