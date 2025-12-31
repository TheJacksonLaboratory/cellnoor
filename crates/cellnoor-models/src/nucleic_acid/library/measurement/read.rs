#[cfg(feature = "app")]
use cellnoor_schema::library_measurements;
use macro_attributes::select;
use uuid::Uuid;

use crate::nucleic_acid::library::measurement::common::LibraryMeasurementFields;

#[select]
pub struct LibraryMeasurement {
    id: Uuid,
    library_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LibraryMeasurementFields,
}
