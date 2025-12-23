#[cfg(feature = "app")]
use cellnoor_schema::specimen_measurements;
use jiff::Timestamp;
use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty::NonEmptyString;
use ranged::RangedF32;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = specimen_measurements))]
pub struct SpecimenMeasurementFields {
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    measured_at: Timestamp,
    data: SpecimenMeasurementData,
}

impl SpecimenMeasurementFields {
    #[must_use]
    pub fn measured_at(&self) -> Timestamp {
        self.measured_at
    }

    #[must_use]
    pub fn data(&self) -> &SpecimenMeasurementData {
        &self.data
    }
}

#[json]
pub enum SpecimenMeasurementData {
    #[serde(rename = "DV200")]
    Dv200 {
        instrument_name: Option<NonEmptyString>,
        value: RangedF32<0, 1>,
    },
    #[serde(rename = "RIN")]
    Rin {
        instrument_name: Option<NonEmptyString>,
        value: RangedF32<1, 10>,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for SpecimenMeasurementData {}
impl_json_from_sql!(SpecimenMeasurementData);

#[cfg(feature = "app")]
impl JsonToSql for SpecimenMeasurementData {}
impl_json_to_sql!(SpecimenMeasurementData);
