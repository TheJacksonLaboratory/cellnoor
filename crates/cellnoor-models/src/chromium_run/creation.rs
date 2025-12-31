use jiff::Timestamp;
use macro_attributes::base_model;
use non_empty::NonEmptyVec;

use crate::chromium_run::common::ChromiumRunFields;

mod ocm;
mod pool_multiplex;
mod singleplex;

pub use ocm::{MAX_SUSPENSIONS_PER_OCM_GEM_POOL, OcmBarcodeId, OcmChipLoading, OcmGemPool};
pub use pool_multiplex::{PoolMultiplexChipLoading, PoolMultiplexGemPool};
pub use singleplex::{SingleplexChipLoading, SingleplexGemPool};

pub const MAX_GEM_POOLS_PER_OCM_RUN: usize = 2;
pub const MAX_GEM_POOLS_PER_NON_OCM_RUN: usize = 8;

#[base_model]
#[derive(serde::Deserialize)]
#[serde(tag = "plexy", rename_all = "snake_case")]
pub enum ChromiumRunCreation {
    OnChipMultiplexing {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gem_pools: NonEmptyVec<OcmGemPool, MAX_GEM_POOLS_PER_OCM_RUN>,
    },
    PoolMultiplex {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gem_pools: NonEmptyVec<PoolMultiplexGemPool, MAX_GEM_POOLS_PER_NON_OCM_RUN>,
    },
    Singleplex {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gem_pools: NonEmptyVec<SingleplexGemPool, MAX_GEM_POOLS_PER_NON_OCM_RUN>,
    },
}

impl ChromiumRunCreation {
    #[must_use]
    pub fn run_at(&self) -> Timestamp {
        let inner = match self {
            Self::OnChipMultiplexing {
                inner,
                gem_pools: _,
            }
            | Self::PoolMultiplex {
                inner,
                gem_pools: _,
            }
            | Self::Singleplex {
                inner,
                gem_pools: _,
            } => inner,
        };

        inner.run_at
    }
}
