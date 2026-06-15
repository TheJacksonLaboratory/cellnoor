use macro_attributes::base_model;
use nonempty::{NonemptyBoundedVec, NonemptyString};

use crate::chromium_run::creation::{
    LoadedEntity,
    ocm::{MAX_SUSPENSIONS_PER_OCM_GEM_WELL, OcmLoadedEntity},
};

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum NewMixedChipLoading {
    Ocm(NonemptyBoundedVec<OcmLoadedEntity, MAX_SUSPENSIONS_PER_OCM_GEM_WELL>),
    Standard(LoadedEntity),
}

#[base_model]
pub struct NewMixedGemWell {
    pub readable_id: NonemptyString,
    pub loading: NewMixedChipLoading,
}
