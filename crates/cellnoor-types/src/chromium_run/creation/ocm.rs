use macro_attributes::{base_model, unit_enum};
use nonempty::{NonemptyBoundedVec, NonemptyString};
use uuid::Uuid;

use crate::chromium_run::creation::NewChipLoadingCommonFields;

pub const MAX_SUSPENSIONS_PER_OCM_GEM_WELL: usize = 4;

#[unit_enum]
pub enum OcmBarcodeId {
    Ob1,
    Ob2,
    Ob3,
    Ob4,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged, rename_all = "snake_case"))]
pub enum NewOcmChipLoading {
    Suspension {
        suspension_id: Uuid,
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChipLoadingCommonFields,
        ocm_barcode_id: OcmBarcodeId,
    },
    SuspensionPool {
        suspension_pool_id: Uuid,
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChipLoadingCommonFields,
        ocm_barcode_id: OcmBarcodeId,
    },
}

#[base_model]
pub struct NewOcmGemWell {
    pub readable_id: NonemptyString,
    pub loading: NonemptyBoundedVec<NewOcmChipLoading, MAX_SUSPENSIONS_PER_OCM_GEM_WELL>,
}
