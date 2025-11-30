use scamplers_models::suspension::measurement::{
    CellSuspensionMeasurementCreation, NucleusSuspensionMeasurementCreation,
    SuspensionMeasurementData, SuspensionMeasurementFields,
};

use crate::validate::{Validate, common::InvalidMeasurement};

const MAX_VIABILITY: i32 = 1;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "typescript",
    ts(rename = "SpecimenMeasurementValidationError")
)]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error(transparent)]
pub enum Error {
    InvalidViability(InvalidMeasurement<0, MAX_VIABILITY>),
}

impl Error {
    fn invalid_viability(viability: f32) -> Self {
        Self::InvalidViability(InvalidMeasurement::new(viability))
    }
}

impl Validate for CellSuspensionMeasurementCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        self.0.validate(db_conn)
    }
}

impl Validate for NucleusSuspensionMeasurementCreation {
    fn validate(&self, db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        self.0.validate(db_conn)
    }
}

impl<C> Validate for SuspensionMeasurementFields<C> {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        match self.data() {
            SuspensionMeasurementData::Concentration {
                inner: _,
                numerator_unit: _,
            }
            | SuspensionMeasurementData::MeanDiameter {
                inner: _,
                object: _,
            }
            | SuspensionMeasurementData::Volume(_) => Ok(()),
            SuspensionMeasurementData::Viability(v) => {
                let value = v.value();
                if value > MAX_VIABILITY as f32 {
                    return Err(Error::invalid_viability(value))?;
                }
                Ok(())
            }
        }
    }
}
