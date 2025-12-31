use cellnoor_models::{specimen::SpecimenId, suspension::SuspensionCreation};
use diesel::PgConnection;
use jiff::Timestamp;

use crate::{
    db::Operation,
    validate::{Validate, common::validate_timestamps},
};

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
            validate_specimen_received_before_suspension_created(
                self.parent_specimen_id(),
                created_at,
                db_conn,
            )?;
        }

        Ok(())
    }
}

fn validate_specimen_received_before_suspension_created(
    specimen_id: impl Into<SpecimenId>,
    created_at: Timestamp,
    db_conn: &mut PgConnection,
) -> Result<(), super::Error> {
    let specimen_received_at = specimen_id.into().execute(db_conn)?.received_at();

    validate_timestamps(specimen_received_at, created_at, "created_at")?;

    Ok(())
}
