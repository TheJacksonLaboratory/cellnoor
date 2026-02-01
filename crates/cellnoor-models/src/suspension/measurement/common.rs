#[cfg(feature = "app")]
use cellnoor_schema::suspension_measurements;
use jiff::Timestamp;
use macro_attributes::{insert_select, json, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql, impl_json_from_sql, impl_json_to_sql};
use ranged::RangedF32;
use uuid::Uuid;

#[cfg(any(feature = "app", feature = "typescript"))]
use crate::suspension::SuspensionContent;
use crate::units::{Microliter, Micrometer, Milliliter};
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql, JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_measurements))]
pub struct SuspensionMeasurementFields {
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    measured_at: Timestamp,
    data: SuspensionMeasurementData,
}

impl SuspensionMeasurementFields {
    pub fn measured_at(&self) -> Timestamp {
        self.measured_at
    }

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
        numerator_unit: SuspensionContent,
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
        object: SuspensionContent,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for SuspensionMeasurementData {}
impl_json_from_sql!(SuspensionMeasurementData);

#[cfg(feature = "app")]
impl JsonToSql for SuspensionMeasurementData {}
impl_json_to_sql!(SuspensionMeasurementData);

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
impl TryFrom<SuspensionContent> for Cells {
    type Error = ();

    fn try_from(value: SuspensionContent) -> Result<Self, Self::Error> {
        match value {
            SuspensionContent::Cells => Ok(Self::Cells),
            SuspensionContent::Nuclei => Err(()),
        }
    }
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
impl TryFrom<SuspensionContent> for Nuclei {
    type Error = ();

    fn try_from(value: SuspensionContent) -> Result<Self, Self::Error> {
        match value {
            SuspensionContent::Cells => Err(()),
            SuspensionContent::Nuclei => Ok(Self::Nuclei),
        }
    }
}

#[cfg(feature = "app")]
impl EnumFromSql for Nuclei {}
impl_enum_from_sql!(Nuclei);

#[cfg(feature = "app")]
impl EnumToSql for Nuclei {}
impl_enum_to_sql!(Nuclei);
