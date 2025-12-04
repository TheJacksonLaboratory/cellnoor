use diesel::prelude::*;
use positive::PositiveU32;
use scamplers_models::{cdna::CdnaCreation, tenx_assay::LibraryType};
use scamplers_schema::library_type_specifications;
use uuid::Uuid;

use crate::{db, validate::Validate};

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "CdnaValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("library type does not exist in assay {assay_id}")]
    NonExistentAssayLibraryType { assay_id: Uuid },
    #[error("wrong volume found")]
    Volume {
        assay_id: Uuid,
        library_type: LibraryType,
        expected: PositiveU32,
        found: PositiveU32,
    },
}

impl Validate for CdnaCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        use library_type_specifications as t;

        let library_type = self.library_type();
        let assay_id = self.assay_id();
        let expected = t::table
            .filter(t::assay_id.eq(assay_id))
            .filter(t::library_type.eq(library_type))
            .select(t::cdna_volume_l)
            .first(db_conn)
            .optional()
            .map_err(db::Error::from)?
            .ok_or(Error::NonExistentAssayLibraryType { assay_id })?;

        let found = self.volume_µl();
        if found != expected {
            return Err(Error::Volume {
                assay_id,
                library_type,
                expected,
                found,
            })?;
        }

        Ok(())
    }
}
