use jiff::Timestamp;
use macro_attributes::{base_model, select};
#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use postgres_types::Json;
use uuid::Uuid;

use crate::suspension::measurement::SuspensionMeasurementQuantity;

pub type SuspensionPoolMeasurementData = SuspensionMeasurementQuantity;

#[base_model]
pub struct NewSuspensionPoolMeasurement {
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    #[cfg_attr(
        all(feature = "postgres-types", feature = "schemars"),
        schemars(with = "SuspensionPoolMeasurementData")
    )]
    pub data: Json<SuspensionPoolMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: SuspensionPoolMeasurementData,
}

#[select]
#[cfg_attr(
    feature = "postgres-types",
    postgres(name = "suspension_pool_measurement")
)]
pub struct SuspensionPoolMeasurement {
    pub id: Uuid,
    pub pool_id: Uuid,
    pub measured_by: Uuid,
    pub measured_at: Timestamp,
    #[cfg(all(feature = "postgres-types", feature = "schemars"))]
    #[cfg_attr(
        all(feature = "postgres-types", feature = "schemars"),
        schemars(with = "SuspensionPoolMeasurementData")
    )]
    pub data: Json<SuspensionPoolMeasurementData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: SuspensionPoolMeasurementData,
}

