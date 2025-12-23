use cellnoor_models::nucleic_acid_measurement::NucleicAcidMeasurementData;

use crate::validate::Validate;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "NucleicAcidMeasurementError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("electrophoretic sizing range minimum must be <= maximum")]
    ElectrophoreticMeasurementSizingRange { min: u16, max: u16 },
}

impl Validate for NucleicAcidMeasurementData {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), crate::validate::Error> {
        match self {
            NucleicAcidMeasurementData::Electrophoretic {
                instrument_name: _,
                mean_size_bp: _,
                sizing_range: (min, max),
                concentration: _,
            } => {
                if min > max {
                    return Err(Error::ElectrophoreticMeasurementSizingRange {
                        min: *min,
                        max: *max,
                    })?;
                }
                Ok(())
            }
            NucleicAcidMeasurementData::Fluorometric {
                instrument_name: _,
                concentration: _,
            } => Ok(()),
        }
    }
}
