#[cfg(feature = "app")]
use cellnoor_schema::chip_loadings;
use macro_attributes::{base_model, insert, simple_enum};
use macros::impl_enum_to_sql;
use non_empty::NonEmptyVec;
use uuid::Uuid;

use crate::chromium_run::common::{ChipLoadingFields, GemPoolFields};
#[cfg(feature = "app")]
use crate::utils::EnumToSql;

pub const MAX_SUSPENSIONS_PER_OCM_GEM_POOL: usize = 4;

#[simple_enum]
#[derive(strum::VariantArray)]
pub enum OcmBarcodeId {
    Ob1,
    Ob2,
    Ob3,
    Ob4,
}

#[cfg(feature = "app")]
impl EnumToSql for OcmBarcodeId {}
impl_enum_to_sql!(OcmBarcodeId);

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings))]
pub struct OcmChipLoading {
    suspension_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChipLoadingFields,
    ocm_barcode_id: OcmBarcodeId,
}

impl OcmChipLoading {
    #[must_use]
    pub fn suspension_id(&self) -> Uuid {
        self.suspension_id
    }
}

#[base_model]
#[derive(serde::Deserialize)]
pub struct OcmGemPool {
    #[serde(flatten)]
    pub inner: GemPoolFields,
    pub loading: NonEmptyVec<OcmChipLoading, MAX_SUSPENSIONS_PER_OCM_GEM_POOL>,
}
