use diesel::prelude::*;
use jiff::Timestamp;
use scamplers_models::{library::LibraryCreation, tenx_assay::LibraryType};
use scamplers_schema::{chromium_runs, library_type_specifications as lib_specs};
use uuid::Uuid;

use crate::{
    db::{self},
    validate::{Validate, cdna::cdna_to_library_spec, common::validate_timestamps},
};

pub mod measurement;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "CdnaValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("wrong volume found")]
    Volume {
        assay_id: Uuid,
        library_type: LibraryType,
        expected: i32,
        found: i32,
    },
}

impl Validate for LibraryCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        let cdna_id = self.cdna_id();
        let (cdna_prepared_at, assay_id, library_type, expected_library_volume) =
            fetch_cdna_data(cdna_id, db_conn)?;

        validate_volume(
            assay_id,
            library_type,
            self.volume_µl(),
            expected_library_volume,
        )?;
        validate_cdna_created_before_library(cdna_prepared_at.to_jiff(), self.prepared_at())?;

        Ok(())
    }
}

fn validate_volume(
    assay_id: Uuid,
    library_type: LibraryType,
    volume: impl Into<i32>,
    expected: i32,
) -> Result<(), super::Error> {
    let volume = volume.into();

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

fn validate_cdna_created_before_library(
    cdna_prepared_at: Timestamp,
    library_prepared_at: Timestamp,
) -> Result<(), super::Error> {
    validate_timestamps(cdna_prepared_at, library_prepared_at, "prepared_at")?;

    Ok(())
}

fn fetch_cdna_data(
    cdna_id: Uuid,
    db_conn: &mut diesel::PgConnection,
) -> Result<(jiff_diesel::Timestamp, Uuid, LibraryType, i32), db::Error> {
    use scamplers_schema::{cdna, library_type_specifications as specs};

    Ok(cdna_to_library_spec()
        .filter(cdna::id.eq(cdna_id))
        .filter(specs::assay_id.eq(chromium_runs::assay_id))
        .filter(specs::library_type.eq(cdna::library_type))
        .select((
            cdna::prepared_at,
            chromium_runs::assay_id,
            lib_specs::library_type,
            lib_specs::library_volume_l,
        ))
        .first(db_conn)?)
}

#[cfg(test)]
mod tests {
    use deadpool_diesel::postgres::Connection;
    use diesel::prelude::*;
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use scamplers_models::tenx_assay::{
        LibraryType, SampleMultiplexing, TenxAssayFilter, TenxAssayQuery,
    };
    use scamplers_schema::{cdna, tenx_assays};
    use uuid::Uuid;

    use crate::{
        db::Operation,
        test_state::{Database, database, root_db_conn},
        validate::{cdna::cdna_to_library_spec, library::fetch_cdna_data},
    };

    #[rstest]
    #[awt]
    #[tokio::test]
    async fn correct_library_spec(
        #[future] root_db_conn: Connection,
        // This argument is required so that the test waits until the database is populated
        #[future] _database: &'static Database,
    ) {
        let three_prime_gex_query = TenxAssayQuery::builder()
            .filter(
                TenxAssayFilter::builder()
                    .names(["Universal 3' Gene Expression".to_owned()])
                    .sample_multiplexing([SampleMultiplexing::Singleplex])
                    .chemistry_versions(["v4 - GEM-X".to_owned()])
                    .library_types([vec![LibraryType::GeneExpression]])
                    .build(),
            )
            .build();

        let three_prime_gex_assay_id = root_db_conn
            .interact(|db_conn| three_prime_gex_query.execute(db_conn).unwrap())
            .await
            .unwrap();
        assert_eq!(three_prime_gex_assay_id.len(), 1);
        let three_prime_gex_assay_id = three_prime_gex_assay_id[0].id();

        let q = cdna_to_library_spec()
            .filter(tenx_assays::id.eq(three_prime_gex_assay_id))
            .select(cdna::id);
        let cdna_id: Uuid = root_db_conn
            .interact(move |db_conn| q.first(db_conn))
            .await
            .unwrap()
            .unwrap();

        let (_, assay_id, library_type, volume) = root_db_conn
            .interact(move |db_conn| fetch_cdna_data(cdna_id, db_conn))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            (assay_id, library_type, volume),
            (three_prime_gex_assay_id, LibraryType::GeneExpression, 35)
        );
    }
}
