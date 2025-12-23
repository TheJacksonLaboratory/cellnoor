#[cfg(feature = "app")]
use cellnoor_schema::suspension_measurements;
use macro_attributes::select;
use uuid::Uuid;

use crate::suspension::{
    common::SuspensionContent, measurement::common::SuspensionMeasurementFields,
};

#[select]
pub struct SuspensionMeasurement {
    id: Uuid,
    suspension_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionMeasurementFields<SuspensionContent>,
}
