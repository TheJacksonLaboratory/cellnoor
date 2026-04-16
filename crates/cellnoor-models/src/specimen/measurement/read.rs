#[cfg(feature = "app")]
use cellnoor_schema::specimen_measurements;
use macro_attributes::select;
use uuid::Uuid;

use crate::specimen::measurement::common::SpecimenMeasurementFields;

#[select]
pub struct SpecimenMeasurement {
    pub id: Uuid,
    pub specimen_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: SpecimenMeasurementFields,
}
