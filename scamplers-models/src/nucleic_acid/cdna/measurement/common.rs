use jiff::Timestamp;
use macro_attributes::insert_select;
#[cfg(feature = "app")]
use scamplers_schema::cdna_measurements;
use uuid::Uuid;

use crate::nucleic_acid::measurement::NucleicAcidMeasurementData;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = cdna_measurements))]
pub struct CdnaMeasurementFields {
    cdna_id: Uuid,
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    measured_at: Timestamp,
    #[serde(flatten)]
    data: NucleicAcidMeasurementData,
}

impl CdnaMeasurementFields {
    pub fn data(&self) -> &NucleicAcidMeasurementData {
        &self.data
    }
}
