use cellnoor_models::{cdna::CdnaCreation, chromium_run::GemPoolId, tenx_assay::LibraryType};
use cellnoor_schema::{
    cdna, chromium_runs, gem_pools, library_type_specifications as lib_specs, tenx_assays,
};
use diesel::prelude::*;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    db::{self, Operation},
    validate::{Validate, common::validate_timestamps},
};

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
        if let Some(gem_pool_id) = self.gem_pool_id() {
            validate_volume(gem_pool_id, self.library_type(), self.volume_µl(), db_conn)?;
            validate_gem_pool_created_before_cdna(gem_pool_id, self.prepared_at(), db_conn)?;
        }

        Ok(())
    }
}

fn validate_volume(
    gem_pool_id: Uuid,
    library_type: LibraryType,
    volume: impl Into<i32>,
    db_conn: &mut diesel::PgConnection,
) -> Result<(), super::Error> {
    let volume = volume.into();
    let (assay_id, library_type, expected) = fetch_cdna_spec(gem_pool_id, library_type, db_conn)?;

    if volume != expected {
        Err(Error::Volume {
            assay_id,
            library_type,
            expected,
            found: volume,
        })?;
    }

    Ok(())
}

fn validate_gem_pool_created_before_cdna(
    gem_pool_id: impl Into<GemPoolId>,
    cdna_created_at: Timestamp,
    db_conn: &mut diesel::PgConnection,
) -> Result<(), super::Error> {
    let gem_pool_creation_time = gem_pool_id.into().execute(db_conn)?.chromium_run_at();
    validate_timestamps(gem_pool_creation_time, cdna_created_at, "prepared_at")?;
    Ok(())
}

fn fetch_cdna_spec(
    gem_pool_id: Uuid,
    library_type: LibraryType,
    db_conn: &mut diesel::PgConnection,
) -> Result<(Uuid, LibraryType, i32), db::Error> {
    Ok(cdna_to_library_spec()
        .filter(lib_specs::library_type.eq(library_type))
        .filter(gem_pools::id.eq(gem_pool_id))
        .select((
            chromium_runs::assay_id,
            lib_specs::library_type,
            lib_specs::cdna_volume_l,
        ))
        .first(db_conn)?)
}

#[diesel::dsl::auto_type]
pub(super) fn cdna_to_library_spec() -> _ {
    cdna::table.inner_join(gem_pools::table.inner_join(
        chromium_runs::table.inner_join(tenx_assays::table.inner_join(lib_specs::table)),
    ))
}
