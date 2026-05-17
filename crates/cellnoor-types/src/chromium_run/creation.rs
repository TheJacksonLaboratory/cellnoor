use macro_attributes::base_model;
use nonempty::NonemptyBoundedVec;
#[cfg(feature = "postgres-types")]
use postgres_types::Json;
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
    #[cfg(not(feature = "postgres-types"))]
    pub suspension_volume_loaded: LoadingVolume,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "LoadingVolume"))]
    pub suspension_volume_loaded: Json<LoadingVolume>,
    #[cfg(not(feature = "postgres-types"))]
    pub buffer_volume_loaded: LoadingVolume,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "LoadingVolume"))]
    pub buffer_volume_loaded: Json<LoadingVolume>,
    pub additional_data: Option<Value>,
}

#[base_model]
#[serde(tag = "plexy", rename_all = "snake_case")]
pub enum NewChromiumRun {
    Mixed {
        #[serde(flatten)]
        common: NewChromiumRunRecord,
        gem_pools: NonemptyBoundedVec<NewMixedGemPool, MAX_GEM_POOLS_PER_OCM_RUN>,
    },
    OnChipMultiplexing {
        #[serde(flatten)]
        common: NewChromiumRunRecord,
        gem_pools: NonemptyBoundedVec<NewOcmGemPool, MAX_GEM_POOLS_PER_OCM_RUN>,
    },
    Standard {
        #[serde(flatten)]
        common: NewChromiumRunRecord,
        gem_pools: NonemptyBoundedVec<NewStandardGemPool, MAX_GEM_POOLS_PER_NON_OCM_RUN>,
    },
}
