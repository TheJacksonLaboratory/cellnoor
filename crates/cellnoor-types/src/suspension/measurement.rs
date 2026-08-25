use macro_attributes::{base_model, unit_enum};
use positive::{PositiveBoundedF32, PositiveF32};

use crate::{
    id::{Id, NoId},
    suspension::{SuspensionContent, measurement::record::SuspensionMeasurementRecord},
    units::{Microliter, Micrometer, Milliliter},
};

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    use postgres_types::Json;
    use uuid::Uuid;

    use crate::suspension::measurement::SuspensionMeasurementData;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "suspension_measurement"))]
    pub struct SuspensionMeasurementRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub measured_by: Uuid,
        pub measured_at: Timestamp,
        #[cfg(all(feature = "postgres-types", feature = "schemars"))]
        #[cfg_attr(
            all(feature = "postgres-types", feature = "schemars"),
            schemars(with = "SuspensionMeasurementData")
        )]
        pub data: Json<SuspensionMeasurementData>,
        #[cfg(not(feature = "postgres-types"))]
        pub data: SuspensionMeasurementData,
    }
}

pub type NewSuspensionMeasurement = SuspensionMeasurementRecord<NoId>;

pub type SuspensionMeasurement = SuspensionMeasurementRecord<Id>;

#[base_model]
#[derive(Copy)]
pub struct SuspensionMeasurementData {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub quantity: SuspensionMeasurementQuantity,
    pub post_hybridization: bool,
}

#[base_model]
#[derive(Copy)]
#[cfg_attr(feature = "serde", serde(tag = "quantity", rename_all = "snake_case"))]
pub enum SuspensionMeasurementQuantity {
    Concentration(SuspensionConcentration),
    Viability(CellViability),
    Volume(SuspensionVolume),
    MeanDiameter(MeanDiameter),
}

#[base_model]
#[derive(Copy)]
pub struct SuspensionConcentration {
    pub counting_method: Option<CountingMethod>,
    pub value: u32,
    pub numerator_unit: SuspensionContent,
    pub denominator_unit: Milliliter,
}

#[base_model]
#[derive(Copy)]
pub struct CellViability {
    pub value: PositiveBoundedF32<1>,
}

#[base_model]
#[derive(Copy)]
pub struct SuspensionVolume {
    pub value: PositiveF32,
    pub unit: Microliter,
}

#[base_model]
#[derive(Copy)]
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
