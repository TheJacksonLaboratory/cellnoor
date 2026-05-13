use jiff::Timestamp;
use macro_attributes::{base_model, select, unit_enum};
use positive::{PositiveBoundedF32, PositiveF32};
#[cfg(feature = "postgres-types")]
use postgres_types::Json;
use uuid::Uuid;

use crate::{
    suspension::SuspensionContent,
    units::{Microliter, Micrometer, Milliliter},
};

#[base_model]
pub struct NewSuspensionMeasurement {
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "SuspensionMeasurementData"))]
    pub data: Json<SuspensionMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: SuspensionMeasurementData,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "suspension_measurement"))]
pub struct SuspensionMeasurement {
    pub id: Uuid,
    pub suspension_id: Uuid,
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "SuspensionMeasurementData"))]
    pub data: Json<SuspensionMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: SuspensionMeasurementData,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "quantity"))]
pub enum SuspensionMeasurementData {
    Concentration {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: Concentration,
        post_hybridization: bool,
    },
    Viability {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: Viability,
        post_hybridization: bool,
    },
    Volume {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: Volume,
        post_hybridization: bool,
    },
    MeanDiameter {
        #[cfg_attr(feature = "serde", serde(flatten))]
        inner: MeanDiameter,
        post_hybridization: bool,
    },
}

#[base_model]
#[cfg_attr(feature = "schemars", schemars(rename = "SuspensionConcentration"))]
pub struct Concentration {
    pub counting_method: Option<CountingMethod>,
    pub value: u32,
    pub numerator_unit: SuspensionContent,
    pub denominator_unit: Milliliter,
}

#[base_model]
pub struct Viability {
    pub value: PositiveBoundedF32<1>,
}

#[base_model]
#[cfg_attr(feature = "schemars", schemars(rename = "SuspensionVolume"))]
pub struct Volume {
    pub value: PositiveF32,
    pub unit: Microliter,
}

#[base_model]
pub struct MeanDiameter {
    pub value: PositiveF32,
    pub object: SuspensionContent,
    pub unit: Micrometer,
}

#[unit_enum]
pub enum CountingMethod {
    BrightField,
    AcridineOrangePropidiumIodide,
    TrypanBlue,
}
