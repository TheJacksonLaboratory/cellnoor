use diesel::prelude::*;
use scamplers_models::{cdna::CdnaCreation, tenx_assay::LibraryType};
use scamplers_schema::{
    cdna, chromium_runs, gem_pools, library_type_specifications as lib_specs, tenx_assays,
};
use uuid::Uuid;

use crate::validate::Validate;

pub mod measurement;

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
        expected: i32,
        found: i32,
    },
}

impl Validate for CdnaCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        let Some(gem_pool_id) = self.gem_pool_id() else {
            return Ok(());
        };

        let library_type = self.library_type();
        let (assay_id, library_type, expected) = cdna_to_library_spec()
            .filter(lib_specs::library_type.eq(library_type))
            .filter(gem_pools::id.eq(gem_pool_id))
            .select((
                chromium_runs::assay_id,
                lib_specs::library_type,
                lib_specs::cdna_volume_l,
            ))
            .first(db_conn)?;

        let found = self.volume_µl().into();
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

#[diesel::dsl::auto_type]
pub(super) fn cdna_to_library_spec() -> _ {
    cdna::table.inner_join(gem_pools::table.inner_join(
        chromium_runs::table.inner_join(tenx_assays::table.inner_join(lib_specs::table)),
    ))
}
