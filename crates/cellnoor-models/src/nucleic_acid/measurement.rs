use macro_attributes::json;
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty::NonEmptyString;

use crate::units::{Microliter, Nanogram, Picogram};
#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[json]
#[cfg_attr(feature = "typescript", ts(concrete(N = String)))]
pub struct Concentration<N> {
    value: u32,
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    numerator_unit: N,
    denominator_unit: Microliter,
}

#[json]
#[serde(tag = "type")]
pub enum NucleicAcidMeasurementData {
    Electrophoretic {
        instrument_name: NonEmptyString,
        mean_size_bp: Option<u16>,
        sizing_range: (u16, u16),
        concentration: Concentration<Picogram>,
    },
    Fluorometric {
        instrument_name: NonEmptyString,
        concentration: Concentration<Nanogram>,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for NucleicAcidMeasurementData {}
impl_json_from_sql!(NucleicAcidMeasurementData);

#[cfg(feature = "app")]
impl JsonToSql for NucleicAcidMeasurementData {}
impl_json_to_sql!(NucleicAcidMeasurementData);
