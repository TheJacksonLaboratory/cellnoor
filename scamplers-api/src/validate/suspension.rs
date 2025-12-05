use jiff::Timestamp;
use scamplers_models::suspension::SuspensionCreation;

use crate::validate::Validate;

pub mod measurement;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "SuspensionValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("suspension cannot be created before its parent specimen is received")]
    CreatedBeforeSpecimenReceived {
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        created_at: Timestamp,
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        specimen_received_at: Timestamp,
    },
}

impl Validate for SuspensionCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        if let Some(created_at) = self.created_at() {
            // validate_specimen_received_before_suspension_created(
            //     self.specimen_id(),
            //     created_at,
            //     db_conn,
            // )?;
        }

        Ok(())
    }
}

// fn validate_specimen_received_before_suspension_created(
//     specimen_id: impl Into<SpecimenId>,
//     created_at: Timestamp,
//     db_conn: &mut PgConnection,
// ) -> Result<(), super::Error> {
//     let specimen_received_at =
// specimen_id.into().execute(db_conn)?.received_at();

//     if specimen_received_at > created_at {
//         return Err(Error::CreatedBeforeSpecimenReceived {
//             created_at,
//             specimen_received_at,
//         })?;
//     }

//     Ok(())
// }
