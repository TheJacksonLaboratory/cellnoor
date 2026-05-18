use macro_attributes::base_model;
use nonempty::{NonemptyBoundedVec, NonemptyString};

use crate::chromium_run::creation::{
    ocm::{MAX_SUSPENSIONS_PER_OCM_GEM_WELL, NewOcmChipLoading},
    standard::NewStandardChipLoading,
};

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged, rename_all = "snake_case"))]
pub enum NewMixedChipLoading {
    Ocm(NewOcmChipLoading),
    Standard(NewStandardChipLoading),
}

#[base_model]
pub struct NewMixedGemWell {
    pub readable_id: NonemptyString,
    pub loading: NonemptyBoundedVec<NewMixedChipLoading, MAX_SUSPENSIONS_PER_OCM_GEM_WELL>,
}
