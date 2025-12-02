use jiff::Timestamp;
use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
#[cfg(feature = "app")]
use scamplers_schema::suspension_pool_measurements;
#[cfg(feature = "app")]
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::suspension::{
    SuspensionContent,
    measurement::common::{Cells, Concentration, MeanDiameter, Nuclei, Viability, Volume},
};
#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[json]
#[serde(tag = "quantity")]
#[cfg_attr(feature = "typescript", ts(concrete(C = SuspensionContent)))]
pub enum SuspensionPoolMeasurementData<C> {
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

#[cfg(feature = "app")]
impl<C> JsonFromSql for SuspensionPoolMeasurementData<C> where C: DeserializeOwned {}
impl_json_from_sql!(SuspensionPoolMeasurementData<SuspensionContent>);

#[cfg(feature = "app")]
impl<C> JsonToSql for SuspensionPoolMeasurementData<C> where C: Serialize {}
impl_json_to_sql!(SuspensionPoolMeasurementData<Cells>);
impl_json_to_sql!(SuspensionPoolMeasurementData<Nuclei>);

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_pool_measurements))]
#[cfg_attr(feature = "typescript", ts(concrete(C = SuspensionContent)))]
pub struct SuspensionPoolMeasurementFields<C> {
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(
        serialize_as = jiff_diesel::Timestamp,
        deserialize_as = jiff_diesel::Timestamp
    ))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    measured_at: Timestamp,
    data: SuspensionPoolMeasurementData<C>,
}

impl<C> SuspensionPoolMeasurementFields<C> {
    pub fn data(&self) -> &SuspensionPoolMeasurementData<C> {
        &self.data
    }
}
