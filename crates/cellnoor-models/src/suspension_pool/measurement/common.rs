#[cfg(feature = "app")]
use cellnoor_schema::suspension_pool_measurements;
use jiff::Timestamp;
use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
use uuid::Uuid;

use crate::suspension::{
    SuspensionContent,
    measurement::common::{Concentration, MeanDiameter, Viability, Volume},
};
#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_pool_measurements))]
pub struct SuspensionPoolMeasurementFields {
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    measured_at: Timestamp,
    data: SuspensionPoolMeasurementData,
}

impl SuspensionPoolMeasurementFields {
    #[must_use]
    pub fn measured_at(&self) -> Timestamp {
        self.measured_at
    }

    #[must_use]
    pub fn data(&self) -> &SuspensionPoolMeasurementData {
        &self.data
    }
}

#[json]
#[serde(tag = "quantity")]
pub enum SuspensionPoolMeasurementData {
    Concentration {
        #[serde(flatten)]
        inner: Concentration,
        numerator_unit: SuspensionContent,
    },
    Viability(Viability),
    Volume(Volume),
    MeanDiameter {
        #[serde(flatten)]
        inner: MeanDiameter,
        object: SuspensionContent,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for SuspensionPoolMeasurementData {}
impl_json_from_sql!(SuspensionPoolMeasurementData);

#[cfg(feature = "app")]
impl JsonToSql for SuspensionPoolMeasurementData {}
impl_json_to_sql!(SuspensionPoolMeasurementData);
