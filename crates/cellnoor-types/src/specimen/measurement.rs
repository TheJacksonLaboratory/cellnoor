use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
use positive::PositiveF32;
#[cfg(feature = "postgres-types")]
use postgres_types::Json;
use uuid::Uuid;

#[base_model]
pub struct NewSpecimenMeasurement {
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "SpecimenMeasurementData"))]
    pub data: Json<SpecimenMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: SpecimenMeasurementData,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_measurement"))]
pub struct SpecimenMeasurement {
    pub id: Uuid,
    pub specimen_id: Uuid,
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "SpecimenMeasurementData"))]
    pub data: Json<SpecimenMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: SpecimenMeasurementData,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "quantity"))]
pub enum SpecimenMeasurementData {
    #[cfg_attr(feature = "serde", serde(rename = "DV200"))]
    Dv200 {
        instrument_name: Option<NonemptyString>,
        value: PositiveF32,
    },
    #[cfg_attr(feature = "serde", serde(rename = "RIN"))]
    Rin {
        instrument_name: Option<NonemptyString>,
        value: PositiveF32,
    },
}
