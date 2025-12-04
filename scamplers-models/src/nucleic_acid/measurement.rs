use macro_attributes::json;
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty::NonEmptyString;
use positive::{PositiveF32, PositiveU32};

#[cfg(feature = "app")]
use crate::utils::JsonToSql;
use crate::{
    units::{Microliter, Nanogram, Picogram},
    utils::JsonFromSql,
};

#[json]
#[cfg_attr(feature = "typescript", ts(concrete(N = String)))]
pub struct Concentration<N> {
    value: PositiveF32,
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    numerator_unit: N,
    denominator_unit: Microliter,
}

#[json]
#[serde(tag = "type")]
pub enum MeasurementData {
    Electrophoretic {
        instrument_name: NonEmptyString,
        mean_size_bp: Option<PositiveU32>,
        sizing_range: (PositiveU32, PositiveU32),
        concentration: Concentration<Picogram>,
    },
    Fluorometric {
        instrument_name: NonEmptyString,
        concentration: Concentration<Nanogram>,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for MeasurementData {}
impl_json_from_sql!(MeasurementData);

#[cfg(feature = "app")]
impl JsonToSql for MeasurementData {}
impl_json_to_sql!(MeasurementData);
