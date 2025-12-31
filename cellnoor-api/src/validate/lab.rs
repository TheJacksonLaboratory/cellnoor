use cellnoor_models::lab::LabCreation;

use crate::validate::Validate;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "LabValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("{delivery_dir} invalid: {message}")]
    DeliveryDir {
        delivery_dir: String,
        message: String,
    },
}

impl Validate for LabCreation {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        if !self.delivery_dir().is_ascii() {
            return Err(Error::DeliveryDir {
                delivery_dir: self.delivery_dir().to_owned(),
                message: "'delivery_dir' must contain only ASCII characters".to_owned(),
            })?;
        }

        Ok(())
    }
}
