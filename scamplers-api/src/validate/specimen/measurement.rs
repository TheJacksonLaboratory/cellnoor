use positive::PositiveF32;
use scamplers_models::specimen::measurement::{SpecimenMeasurement, SpecimenMeasurementData};

use crate::validate::{Validate, common::InvalidMeasurement};

const MIN_DV200: i32 = 0;
const MAX_DV200: i32 = 1;
const MIN_RIN: i32 = 1;
const MAX_RIN: i32 = 10;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "typescript",
    ts(rename = "SpecimenMeasurementValidationError")
)]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error(transparent)]
pub enum Error {
    InvalidDv200(InvalidMeasurement<MIN_DV200, MAX_DV200>),
    InvalidRin(InvalidMeasurement<MIN_RIN, MAX_RIN>),
}

impl Error {
    fn invalid_dv200(dv200: f32) -> Self {
        Self::InvalidDv200(InvalidMeasurement::new(dv200))
    }

    fn invalid_rin(rin: f32) -> Self {
        Self::InvalidRin(InvalidMeasurement::new(rin))
    }
}

impl Validate for SpecimenMeasurement {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        match self.data() {
            SpecimenMeasurementData::Dv200 {
                instrument_name: _,
                value: PositiveF32(value),
            } if (*value > MAX_DV200 as f32) => Err(Error::invalid_dv200(*value))?,
            SpecimenMeasurementData::Rin {
                instrument_name: _,
                value: PositiveF32(value),
            } if (*value < MIN_RIN as f32) || (*value > MAX_RIN as f32) => {
                Err(Error::invalid_rin(*value))?
            }
            SpecimenMeasurementData::Dv200 { .. } | SpecimenMeasurementData::Rin { .. } => Ok(()),
        }
    }
}
