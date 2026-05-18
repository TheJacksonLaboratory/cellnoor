use jiff::Timestamp;
use macro_attributes::base_model;
use nonempty::NonemptyString;
use positive::PositiveU32;
#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use postgres_types::Json;
use uuid::Uuid;

use crate::units::{Microliter, Nanogram, Picogram};

#[base_model]
pub struct Concentration<N> {
    pub value: PositiveU32,
    pub numerator_unit: N,
    pub denominator_unit: Microliter,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum NucleicAcidMeasurementData {
    Electrophoretic {
        instrument_name: NonemptyString,
        mean_size_bp: Option<PositiveU32>,
        sizing_range: (u16, PositiveU32),
        concentration: Concentration<Picogram>,
    },
    Fluorometric {
        instrument_name: NonemptyString,
        concentration: Concentration<Nanogram>,
    },
}

#[base_model]
pub struct NewNucleicAcidMeasurement {
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    #[cfg_attr(
        all(feature = "postgres-types", feature = "schemars"),
        schemars(with = "NucleicAcidMeasurementData")
    )]
    pub data: Json<NucleicAcidMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: NucleicAcidMeasurementData,
}
