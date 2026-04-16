#[cfg(feature = "app")]
use cellnoor_schema::suspension_measurements;
use macro_attributes::select;
use uuid::Uuid;

use crate::suspension::measurement::common::SuspensionMeasurementFields;

#[select]
pub struct SuspensionMeasurement {
    pub id: Uuid,
    pub suspension_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: SuspensionMeasurementFields,
}
