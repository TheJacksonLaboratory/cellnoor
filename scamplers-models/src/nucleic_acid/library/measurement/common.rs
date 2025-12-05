use jiff::Timestamp;
use macro_attributes::insert_select;
#[cfg(feature = "app")]
use scamplers_schema::library_measurements;
use uuid::Uuid;

use crate::nucleic_acid::measurement::NucleicAcidMeasurementData;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = library_measurements))]
pub struct LibraryMeasurementFields {
    library_id: Uuid,
    measured_by: Uuid,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    measured_at: Timestamp,
    #[serde(flatten)]
    data: NucleicAcidMeasurementData,
}

impl LibraryMeasurementFields {
    pub fn data(&self) -> &NucleicAcidMeasurementData {
        &self.data
    }

    pub fn library_id(&self) -> Uuid {
        self.library_id
    }

    pub fn measured_at(&self) -> Timestamp {
        self.measured_at
    }
}
