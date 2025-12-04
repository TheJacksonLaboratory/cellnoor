use macro_attributes::base_model;
use non_empty::NonEmptyVec;

use crate::chromium_run::common::ChromiumRunFields;

mod ocm;
mod pool_multiplex;
mod singleplex;

pub use ocm::OcmGemPool;
pub use pool_multiplex::PoolMultiplexGemPool;
pub use singleplex::SingleplexGemPool;

const MAX_GEM_POOLS_IN_OCM_RUN: usize = 2;
const MAX_GEM_POOLS_IN_NON_OCM_RUN: usize = 8;

#[base_model]
#[derive(serde::Deserialize)]
#[serde(tag = "plexy", rename_all = "snake_case")]
pub enum ChromiumRunCreation {
    OnChipMultiplexing {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gems: NonEmptyVec<OcmGemPool, MAX_GEM_POOLS_IN_OCM_RUN>,
    },
    PoolMultiplex {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gems: NonEmptyVec<PoolMultiplexGemPool, MAX_GEM_POOLS_IN_NON_OCM_RUN>,
    },
    Singleplex {
        #[serde(flatten)]
        inner: ChromiumRunFields,
        gems: NonEmptyVec<SingleplexGemPool, MAX_GEM_POOLS_IN_NON_OCM_RUN>,
    },
}
