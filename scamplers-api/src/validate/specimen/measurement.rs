use positive::PositiveF32;
use scamplers_models::specimen::measurement::{MeasurementData, SpecimenMeasurement};

use crate::validate::Validate;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "typescript",
    ts(rename = "SpecimenMeasurementValidationError")
)]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("RIN must be between {min} and {max}, found: {found}")]
    InvalidRin {
        min: f32,
        max: f32,
        found: PositiveF32,
    },
    #[error("DV200 must be between {min} and {max}")]
    InvalidDv200 {
        min: f32,
        max: f32,
        found: PositiveF32,
    },
}

const MIN_DV200: f32 = 0.0;
const MAX_DV200: f32 = 1.0;
const MIN_RIN: f32 = 1.0;
const MAX_RIN: f32 = 10.0;

impl Error {
    fn invalid_dv200(dv200: PositiveF32) -> Self {
        Self::InvalidDv200 {
            min: MIN_DV200,
            max: MAX_DV200,
            found: dv200,
        }
    }

    fn invalid_rin(rin: PositiveF32) -> Self {
        Self::InvalidRin {
            min: MIN_RIN,
            max: MAX_RIN,
            found: rin,
        }
    }
}

impl Validate for SpecimenMeasurement {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        match self.data() {
            MeasurementData::Dv200 {
                instrument_name: _,
                value,
            } if (*value > MAX_DV200) => Err(Error::invalid_dv200(*value))?,
            MeasurementData::Rin {
                instrument_name: _,
                value,
            } if (*value < MIN_RIN) || (*value > MAX_RIN) => Err(Error::invalid_rin(*value))?,
            MeasurementData::Dv200 { .. } | MeasurementData::Rin { .. } => Ok(()),
        }
    }
}
