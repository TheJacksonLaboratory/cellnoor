use jiff::Timestamp;
use macro_attributes::select;
#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use postgres_types::Json;
use uuid::Uuid;

use crate::nucleic_acid_measurement::NucleicAcidMeasurementData;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "cdna_measurement"))]
pub struct CdnaMeasurement {
    pub id: Uuid,
    pub cdna_id: Uuid,
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "NucleicAcidMeasurementData"))]
    pub data: Json<NucleicAcidMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: NucleicAcidMeasurementData,
}
