use diesel::PgConnection;

use crate::{db, validate::common::TimestampError};

mod cdna;
mod chromium_dataset;
mod chromium_run;
mod common;
mod initial_data;
mod institution;
mod lab;
mod library;
mod nucleic_acid_measurement;
mod person;
mod sequencing_run;
mod specimen;
mod suspension;
mod suspension_pool;
mod tenx_assay;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "DataValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error(transparent)]
pub enum Error {
    InsertInitialData(#[from] initial_data::Error),
    CreatePerson(#[from] person::Error),
    CreateLab(#[from] lab::Error),
    CreateSpecimen(#[from] specimen::Error),
    CreateSuspension(#[from] suspension::Error),
    CreateSuspensionPool(#[from] suspension_pool::Error),
    CreateCdna(#[from] cdna::Error),
    CreateLibrary(#[from] library::Error),
    CreateNucleicAcidMeasurement(#[from] nucleic_acid_measurement::Error),
    CreateChromiumDataset(#[from] chromium_dataset::Error),
    Timestamp(#[from] TimestampError),
    Database(#[from] db::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Self {
        Self::Database(db::Error::from(error))
    }
}

pub trait Validate {
    fn validate(&self, _db_conn: &mut PgConnection) -> Result<(), Error> {
        Ok(())
    }
}
