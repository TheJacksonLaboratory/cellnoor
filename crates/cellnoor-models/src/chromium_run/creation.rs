use jiff::Timestamp;
use macro_attributes::base_model;
use non_empty::NonEmptyVec;
#[cfg(feature = "app")]
use schemars::JsonSchema;

use crate::chromium_run::common::ChromiumRunFields;

#[cfg(feature = "builder")]
pub mod ocm;
#[cfg(feature = "builder")]
pub mod standard;

#[cfg(not(feature = "builder"))]
mod ocm;
#[cfg(not(feature = "builder"))]
mod standard;

pub use ocm::{MAX_SUSPENSIONS_PER_OCM_GEM_POOL, OcmBarcodeId, OcmChipLoading, OcmGemPool};
pub use standard::{StandardChipLoading, StandardGemPool};

pub const MAX_GEM_POOLS_PER_OCM_RUN: usize = 2;
pub const MAX_GEM_POOLS_PER_NON_OCM_RUN: usize = 8;

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
#[serde(tag = "plexy", rename_all = "snake_case")]
pub enum NewChromiumRun {
    OnChipMultiplexing {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gem_pools: NonEmptyVec<OcmGemPool, MAX_GEM_POOLS_PER_OCM_RUN>,
    },
    Standard {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gem_pools: NonEmptyVec<StandardGemPool, MAX_GEM_POOLS_PER_NON_OCM_RUN>,
    },
}

impl NewChromiumRun {
    #[must_use]
    pub fn run_at(&self) -> Timestamp {
        let (Self::OnChipMultiplexing { inner, .. } | Self::Standard { inner, .. }) = self;

        inner.run_at
    }
}
