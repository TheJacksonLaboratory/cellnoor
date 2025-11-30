use jiff::Timestamp;
use macro_attributes::{insert_select, json, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql, impl_json_from_sql, impl_json_to_sql};
use positive::PositiveF32;
use scamplers_schema::suspension_measurements;
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};
use crate::{
    suspension::SuspensionContent,
    utils::{JsonFromSql, JsonToSql},
};

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
enum Microliter {
    #[serde(alias = "µL")]
    Microliter,
}

#[cfg(feature = "app")]
impl EnumFromSql for Microliter {}
impl_enum_from_sql!(Microliter);

#[cfg(feature = "app")]
impl EnumToSql for Microliter {}
impl_enum_to_sql!(Microliter);

#[simple_enum]
enum Milliliter {
    #[serde(alias = "mL")]
    Milliliter,
}

#[cfg(feature = "app")]
impl EnumFromSql for Milliliter {}
impl_enum_from_sql!(Milliliter);

#[cfg(feature = "app")]
impl EnumToSql for Milliliter {}
impl_enum_to_sql!(Milliliter);

#[simple_enum]
enum Micrometer {
    #[serde(alias = "µm")]
    Micrometer,
}

#[cfg(feature = "app")]
impl EnumFromSql for Micrometer {}
impl_enum_from_sql!(Micrometer);

#[cfg(feature = "app")]
impl EnumToSql for Micrometer {}
impl_enum_to_sql!(Micrometer);

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

#[json]
pub struct Concentration {
    counting_method: Option<CountingMethod>,
    post_hybridization: bool,
    value: PositiveF32,
    denominator_unit: Milliliter,
}

#[json]
pub struct Viability {
    value: PositiveF32,
}

impl Viability {
    pub fn value(&self) -> f32 {
        self.value.0
    }
}

#[json]
pub struct Volume {
    post_hybridization: bool,
    value: PositiveF32,
    unit: Microliter,
}

#[json]
pub struct MeanDiameter {
    post_hybridization: bool,
    value: PositiveF32,
    unit: Micrometer,
}

#[json]
#[serde(tag = "quantity")]
#[cfg_attr(feature = "typescript", ts(concrete(C = SuspensionContent)))]
pub enum SuspensionMeasurementData<C> {
    Concentration {
        #[serde(flatten)]
        inner: Concentration,
        #[cfg_attr(feature = "typescript", ts(as = "SuspensionContent"))]
        numerator_unit: C,
    },
    Viability(Viability),
    Volume(Volume),
    MeanDiameter {
        #[serde(flatten)]
        inner: MeanDiameter,
        #[cfg_attr(feature = "typescript", ts(as = "SuspensionContent"))]
        object: C,
    },
}

impl<C> JsonFromSql for SuspensionMeasurementData<C> where C: DeserializeOwned {}
impl_json_from_sql!(SuspensionMeasurementData<SuspensionContent>);

impl<C> JsonToSql for SuspensionMeasurementData<C> where C: Serialize {}
impl_json_to_sql!(SuspensionMeasurementData<Cells>);
impl_json_to_sql!(SuspensionMeasurementData<Nuclei>);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_measurements))]
#[cfg_attr(feature = "typescript", ts(concrete(C = SuspensionContent)))]
pub struct SuspensionMeasurementFields<C> {
    suspension_id: Uuid,
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
    pub fn data(&self) -> &SuspensionMeasurementData<C> {
        &self.data
    }
}
