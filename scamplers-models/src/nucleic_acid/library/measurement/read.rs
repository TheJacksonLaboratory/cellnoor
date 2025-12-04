use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::library_measurements;
use uuid::Uuid;

use crate::nucleic_acid::library::measurement::common::LibraryMeasurementFields;

#[select]
pub struct LibraryMeasurement {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LibraryMeasurementFields,
}
