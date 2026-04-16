#[cfg(feature = "app")]
use cellnoor_schema::cdna_measurements;
use macro_attributes::select;
use uuid::Uuid;

use crate::nucleic_acid::cdna::measurement::common::CdnaMeasurementFields;

#[select]
pub struct CdnaMeasurement {
    pub id: Uuid,
    pub cdna_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: CdnaMeasurementFields,
}
