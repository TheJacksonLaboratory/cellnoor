use cellnoor_models::chromium_dataset::{ChromiumDatasetCmdline, ChromiumDatasetCreation};
use cellnoor_schema::{gem_pools, libraries, sequencing_runs, sequencing_submissions, tenx_assays};
use diesel::prelude::*;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    db,
    validate::{Validate, cdna::cdna_to_library_spec},
};

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "ChromiumDatasetValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("all libraries must come from same GEM pool")]
    DifferentGemPools,
    #[error("expected one of the following cmdlines: {expected:?}")]
    Cmdline {
        expected: Vec<ChromiumDatasetCmdline>,
    },
    #[error("data path does not match expected pattern")]
    DataPath,
    #[error("libraries were not sequenced")]
    LibrariesNotSequenced { library_ids: Vec<Uuid> },
}

impl Validate for ChromiumDatasetCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        let library_ids = self.library_ids();
        let validation_data = fetch_validation_data(library_ids, db_conn)?;

        validate_same_gem_pool(&validation_data)?;
        validate_cmdline(&validation_data, self.cmdline())?;
        validate_sequencing_runs_finished(&validation_data, library_ids, self.delivered_at())?;

        Ok(())
    }
}

type ValidationDatum = (Uuid, Option<Timestamp>, Vec<ChromiumDatasetCmdline>);

fn validate_same_gem_pool(validation_data: &[ValidationDatum]) -> Result<(), Error> {
    fn extract_gem_pool_id((gem_pool_id, ..): &ValidationDatum) -> &Uuid {
        gem_pool_id
    }

    let first_gem_pool_id = validation_data.iter().map(extract_gem_pool_id).next();
    if validation_data
        .iter()
        .map(|(gem_pool_id, ..)| Some(gem_pool_id))
        .any(|gem_pool_id| gem_pool_id != first_gem_pool_id)
    {
        return Err(Error::DifferentGemPools);
    }

    Ok(())
}

fn validate_cmdline(
    validation_data: &[ValidationDatum],
    cmdline: ChromiumDatasetCmdline,
) -> Result<(), Error> {
    let expected_cmdlines: Vec<ChromiumDatasetCmdline> = validation_data
        .iter()
        .flat_map(|(_, _, c)| c.clone())
        .collect();

    if !expected_cmdlines.contains(&cmdline) {
        return Err(Error::Cmdline {
            expected: expected_cmdlines,
        });
    }

    Ok(())
}

fn validate_sequencing_runs_finished(
    validation_data: &[ValidationDatum],
    library_ids: &[Uuid],
    dataset_delivered_at: Timestamp,
) -> Result<(), Error> {
    if validation_data.is_empty() {
        return Err(Error::LibrariesNotSequenced {
            library_ids: library_ids.into(),
        });
    }
    let mut sequencing_run_finish_times = validation_data.iter().map(|(_, ts, _)| ts.as_ref());
    if sequencing_run_finish_times.any(|ts| ts > Some(&dataset_delivered_at)) {
        return Err(Error::LibrariesNotSequenced {
            library_ids: library_ids.into(),
        });
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
fn fetch_validation_data(
    library_ids: &[Uuid],
    db_conn: &mut diesel::PgConnection,
) -> Result<Vec<ValidationDatum>, db::Error> {
    let results: Vec<(
        Uuid,
        jiff_diesel::NullableTimestamp,
        Option<Vec<Option<ChromiumDatasetCmdline>>>,
    )> = libraries::table
        .inner_join(cdna_to_library_spec())
        .inner_join(sequencing_submissions::table.inner_join(sequencing_runs::table))
        .filter(libraries::id.eq_any(library_ids))
        .select((
            gem_pools::id,
            sequencing_runs::finished_at,
            tenx_assays::cmdlines,
        ))
        .load(db_conn)?;

    Ok(results
        .into_iter()
        .map(|(id, ts, cmdlines)| {
            (
                id,
                ts.to_jiff(),
                cmdlines.into_iter().flatten().flatten().collect(),
            )
        })
        .collect())
}
