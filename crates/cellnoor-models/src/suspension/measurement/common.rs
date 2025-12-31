#[cfg(feature = "app")]
use cellnoor_schema::suspension_measurements;
use jiff::Timestamp;
use macro_attributes::{insert_select, json, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql, impl_json_from_sql, impl_json_to_sql};
use ranged::RangedF32;
#[cfg(feature = "app")]
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[cfg(any(feature = "app", feature = "typescript"))]
use crate::suspension::SuspensionContent;
use crate::units::{Microliter, Micrometer, Milliliter};
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql, JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_measurements))]
#[cfg_attr(feature = "typescript", ts(concrete(C = SuspensionContent)))]
pub struct SuspensionMeasurementFields<C> {
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    measured_at: Timestamp,
    data: SuspensionMeasurementData<C>,
}

impl<C> SuspensionMeasurementFields<C> {
    pub fn measured_at(&self) -> Timestamp {
        self.measured_at
    }

    pub fn data(&self) -> &SuspensionMeasurementData<C> {
        &self.data
    }
}

#[json]
#[serde(tag = "quantity")]
#[cfg_attr(feature = "typescript", ts(concrete(C = SuspensionContent)))]
pub enum SuspensionMeasurementData<C> {
    Concentration {
        #[serde(flatten)]
        inner: Concentration,
        post_hybridization: bool,
        #[cfg_attr(feature = "typescript", ts(as = "SuspensionContent"))]
        numerator_unit: C,
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
        #[cfg_attr(feature = "typescript", ts(as = "SuspensionContent"))]
        object: C,
    },
}

#[cfg(feature = "app")]
impl<C> JsonFromSql for SuspensionMeasurementData<C> where C: DeserializeOwned {}
impl_json_from_sql!(SuspensionMeasurementData<SuspensionContent>);

#[cfg(feature = "app")]
impl<C> JsonToSql for SuspensionMeasurementData<C> where C: Serialize {}
impl_json_to_sql!(SuspensionMeasurementData<Cells>);
impl_json_to_sql!(SuspensionMeasurementData<Nuclei>);

#[json]
pub struct Concentration {
    counting_method: Option<CountingMethod>,
    value: u32,
    denominator_unit: Milliliter,
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
pub struct Volume {
    value: RangedF32<0, { u32::MAX }>,
    unit: Microliter,
}

#[json]
pub struct MeanDiameter {
    value: RangedF32<0, { u32::MAX }>,
    unit: Micrometer,
}

#[simple_enum]
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

#[simple_enum]
pub enum Cells {
    Cells,
}

#[cfg(feature = "app")]
impl EnumFromSql for Cells {}
impl_enum_from_sql!(Cells);

#[cfg(feature = "app")]
impl EnumToSql for Cells {}
impl_enum_to_sql!(Cells);

#[simple_enum]
pub enum Nuclei {
    Nuclei,
}

#[cfg(feature = "app")]
impl EnumFromSql for Nuclei {}
impl_enum_from_sql!(Nuclei);

#[cfg(feature = "app")]
impl EnumToSql for Nuclei {}
impl_enum_to_sql!(Nuclei);
