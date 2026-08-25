use macro_attributes::{base_model, unit_enum};
use nonempty::{NonemptyBoundedVec, NonemptyString};

use crate::chromium_run::creation::LoadedEntity;

pub const MAX_SUSPENSIONS_PER_OCM_GEM_WELL: usize = 4;

#[unit_enum]
pub enum OcmBarcodeId {
    Ob1,
    Ob2,
    Ob3,
    Ob4,
}

#[base_model]
#[derive(Copy)]
pub struct OcmLoadedEntity {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub loaded_entity: LoadedEntity,
    pub ocm_barcode_id: OcmBarcodeId,
}

#[base_model]
pub struct NewOcmGemWell {
    pub readable_id: NonemptyString,
    pub loading: NonemptyBoundedVec<OcmLoadedEntity, MAX_SUSPENSIONS_PER_OCM_GEM_WELL>,
}
