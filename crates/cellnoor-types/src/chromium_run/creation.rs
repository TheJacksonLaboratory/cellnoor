use macro_attributes::base_model;
use nonempty::NonemptyBoundedVec;
use serde_json::Value;

use crate::chromium_run::{
    LoadingVolume, NewChromiumRunRecord,
    creation::{mixed::NewMixedGemPool, ocm::NewOcmGemPool, standard::NewStandardGemPool},
};

pub mod mixed;
pub mod ocm;
pub mod standard;

pub const MAX_GEM_POOLS_PER_OCM_RUN: usize = 2;
pub const MAX_GEM_POOLS_PER_NON_OCM_RUN: usize = 8;

#[base_model]
pub struct NewChipLoadingCommonFields {
    pub suspension_volume_loaded: LoadingVolume,
    pub buffer_volume_loaded: LoadingVolume,
    pub additional_data: Option<Value>,
}

#[base_model]
#[serde(tag = "plexy", rename_all = "snake_case")]
pub enum NewChromiumRun {
    Mixed {
        #[serde(flatten)]
        inner: NewChromiumRunRecord,
        gem_pools: NonemptyBoundedVec<NewMixedGemPool, MAX_GEM_POOLS_PER_OCM_RUN>,
    },
    OnChipMultiplexing {
        #[serde(flatten)]
        inner: NewChromiumRunRecord,
        gem_pools: NonemptyBoundedVec<NewOcmGemPool, MAX_GEM_POOLS_PER_OCM_RUN>,
    },
    Standard {
        #[serde(flatten)]
        inner: NewChromiumRunRecord,
        gem_pools: NonemptyBoundedVec<NewStandardGemPool, MAX_GEM_POOLS_PER_NON_OCM_RUN>,
    },
}
