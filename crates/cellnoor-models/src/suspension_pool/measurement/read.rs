#[cfg(feature = "app")]
use cellnoor_schema::suspension_pool_measurements;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    suspension::common::SuspensionContent,
    suspension_pool::measurement::SuspensionPoolMeasurementFields,
};

#[select]
pub struct SuspensionPoolMeasurement {
    id: Uuid,
    pool_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionPoolMeasurementFields<SuspensionContent>,
}
