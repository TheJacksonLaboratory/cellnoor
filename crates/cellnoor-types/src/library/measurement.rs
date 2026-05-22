use jiff::Timestamp;
use macro_attributes::{base_model, select};
#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use postgres_types::Json;
use uuid::Uuid;

#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use crate::nucleic_acid_measurement::NucleicAcidMeasurementData;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "library_measurement"))]
pub struct LibraryMeasurement {
    pub id: Uuid,
    pub library_id: Uuid,
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    #[cfg_attr(
        all(feature = "postgres-types", feature = "schemars"),
        schemars(with = "NucleicAcidMeasurementData")
    )]
    pub data: Json<NucleicAcidMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: NucleicAcidMeasurementData,
}

#[base_model]
pub struct NewLibraryMeasurement {
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    #[cfg_attr(
        all(feature = "postgres-types", feature = "schemars"),
        schemars(with = "NucleicAcidMeasurementData")
    )]
    pub data: Json<NucleicAcidMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: NucleicAcidMeasurementData,
}
