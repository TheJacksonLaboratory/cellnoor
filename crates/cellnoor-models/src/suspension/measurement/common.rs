#[cfg(feature = "app")]
use cellnoor_schema::suspension_measurements;
use jiff::Timestamp;
use macro_attributes::{insert_select, json, unit_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql, impl_json_from_sql, impl_json_to_sql};
use ranged::RangedF32;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::suspension::SuspensionContent;
use crate::units::{Microliter, Micrometer, Milliliter};
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql, JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_measurements), schemars(inline))]
pub struct SuspensionMeasurementFields {
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    measured_at: Timestamp,
    data: SuspensionMeasurementData,
}

impl SuspensionMeasurementFields {
    #[must_use]
    pub fn measured_at(&self) -> Timestamp {
        self.measured_at
    }

    #[must_use]
    pub fn data(&self) -> &SuspensionMeasurementData {
        &self.data
    }
}

#[json]
#[serde(tag = "quantity")]
pub enum SuspensionMeasurementData {
    Concentration {
        #[serde(flatten)]
        inner: Concentration,
        post_hybridization: bool,
    },
    Viability {
        #[serde(flatten)]
        inner: Viability,
        post_hybridization: bool,
    },
    Volume {
        #[serde(flatten)]
        inner: Volume,
        post_hybridization: bool,
    },
    MeanDiameter {
        #[serde(flatten)]
        inner: MeanDiameter,
        post_hybridization: bool,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for SuspensionMeasurementData {}
impl_json_from_sql!(SuspensionMeasurementData);

#[cfg(feature = "app")]
impl JsonToSql for SuspensionMeasurementData {}
impl_json_to_sql!(SuspensionMeasurementData);

#[json]
#[cfg_attr(feature = "app", schemars(rename = "SuspensionConcentration"))]
pub struct Concentration {
    counting_method: Option<CountingMethod>,
    value: u32,
    numerator_unit: SuspensionContent,
    denominator_unit: Milliliter,
}

impl Concentration {
    pub fn numerator_unit(&self) -> SuspensionContent {
        self.numerator_unit
    }
}

#[json]
pub struct Viability {
    value: RangedF32<0, 1>,
}

impl Viability {
    pub fn value(&self) -> RangedF32<0, 1> {
        self.value
    }
}

#[json]
#[cfg_attr(feature = "app", schemars(rename = "SuspensionVolume"))]
pub struct Volume {
    value: RangedF32<0, { u32::MAX }>,
    unit: Microliter,
}

#[json]
pub struct MeanDiameter {
    value: RangedF32<0, { u32::MAX }>,
    object: SuspensionContent,
    unit: Micrometer,
}

impl MeanDiameter {
    pub fn object(&self) -> SuspensionContent {
        self.object
    }
}

#[unit_enum]
enum CountingMethod {
    BrightField,
    AcridineOrangePropidiumIodide,
    TrypanBlue,
}

#[cfg(feature = "app")]
impl EnumFromSql for CountingMethod {}
impl_enum_from_sql!(CountingMethod);

#[cfg(feature = "app")]
impl EnumToSql for CountingMethod {}
impl_enum_to_sql!(CountingMethod);
