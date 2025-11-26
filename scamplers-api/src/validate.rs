use diesel::PgConnection;

mod initial_data;
mod institution;
mod lab;
mod person;
mod specimen;
mod suspension;
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
    CreateSpecimenMeasurement(#[from] specimen::measurement::Error),
}

pub trait Validate {
    fn validate(&self, _db_conn: &mut PgConnection) -> Result<(), Error> {
        Ok(())
    }
}
