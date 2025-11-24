use jiff::Timestamp;
use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty_string::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::specimen_measurements;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = specimen_measurements))]
pub struct SpecimenMeasurement {
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(type = "Date"))]
    measured_at: Timestamp,
    data: MeasurementData,
}

impl SpecimenMeasurement {
    #[must_use]
    pub fn data(&self) -> &MeasurementData {
        &self.data
    }
}

#[json]
pub enum MeasurementData {
    #[serde(rename = "RIN")]
    Rin {
        instrument_name: Option<NonEmptyString>,
        value: f32,
    },
    #[serde(rename = "DV200")]
    Dv200 {
        instrument_name: Option<NonEmptyString>,
        value: f32,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for MeasurementData {}
impl_json_from_sql!(MeasurementData);

#[cfg(feature = "app")]
impl JsonToSql for MeasurementData {}
impl_json_to_sql!(MeasurementData);
