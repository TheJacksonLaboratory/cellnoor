#[cfg(feature = "app")]
use cellnoor_schema::chip_loadings;
use macro_attributes::select;
use uuid::Uuid;

use crate::chromium_run::Volume;

#[select]
pub struct ChipLoading {
    pub id: Uuid,
    pub gem_pool_id: Uuid,
    pub suspension_id: Option<Uuid>,
    pub ocm_barcode_id: Option<String>,
    pub suspension_pool_id: Option<Uuid>,
    pub suspension_volume_loaded: Volume,
    pub buffer_volume_loaded: Volume,
    pub additional_data: Option<serde_json::Value>,
}
