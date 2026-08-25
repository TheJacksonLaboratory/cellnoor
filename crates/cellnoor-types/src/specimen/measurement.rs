use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
use positive::PositiveBoundedF32;
#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use postgres_types::Json;
use uuid::Uuid;

#[base_model]
pub struct NewSpecimenMeasurement {
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    #[cfg_attr(
        all(feature = "postgres-types", feature = "schemars"),
        schemars(with = "SpecimenMeasurementData")
    )]
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
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    #[cfg_attr(
        all(feature = "postgres-types", feature = "schemars"),
        schemars(with = "SpecimenMeasurementData")
    )]
    pub data: Json<SpecimenMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: SpecimenMeasurementData,
}

#[base_model]
pub struct SpecimenMeasurementData {
    pub instrument_name: Option<NonemptyString>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub quantity: SpecimenMeasurementQuantity,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "quantity"))]
pub enum SpecimenMeasurementQuantity {
    #[cfg_attr(feature = "serde", serde(rename = "DV200"))]
    Dv200 { value: PositiveBoundedF32<1> },
    #[cfg_attr(feature = "serde", serde(rename = "RIN"))]
    Rin { value: PositiveBoundedF32<10> },
}
