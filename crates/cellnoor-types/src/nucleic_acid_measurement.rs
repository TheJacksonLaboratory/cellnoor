use jiff::Timestamp;
use macro_attributes::base_model;
use nonempty::NonemptyString;
use positive::PositiveI32;
#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use postgres_types::Json;
use uuid::Uuid;

use crate::units::{Microliter, Nanogram, Picogram};

#[base_model]
#[cfg_attr(feature = "schemars", schemars(rename = "{N}Concentration"))]
pub struct Concentration<N> {
    pub value: PositiveI32,
    pub numerator_unit: N,
    pub denominator_unit: Microliter,
}

#[base_model]
pub struct NucleicAcidMeasurementData {
    pub instrument_name: NonemptyString,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub method: NucleicAcidMeasurementMethod,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum NucleicAcidMeasurementMethod {
    Electrophoretic {
        mean_size_bp: Option<PositiveI32>,
        sizing_range: (u16, PositiveI32),
        concentration: Concentration<Picogram>,
    },
    Fluorometric {
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
