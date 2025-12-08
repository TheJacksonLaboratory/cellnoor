use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::specimen_measurements;
use uuid::Uuid;

use crate::specimen::measurement::common::SpecimenMeasurementFields;

#[select]
pub struct SpecimenMeasurement {
    id: Uuid,
    specimen_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SpecimenMeasurementFields,
}
