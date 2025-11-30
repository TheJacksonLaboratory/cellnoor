use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::suspension_measurements;
use uuid::Uuid;

use crate::suspension::{
    common::SuspensionContent, measurement::common::SuspensionMeasurementFields,
};

#[select]
pub struct SuspensionMeasurement {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionMeasurementFields<SuspensionContent>,
}

// pub type SuspensionMeasurement =
// SuspensionMeasurementFields<SuspensionContent>;
