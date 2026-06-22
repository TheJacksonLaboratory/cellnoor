use macro_attributes::base_model;
use nonempty::NonemptyBoundedVec;
use uuid::Uuid;

use crate::{
    chromium_run::{
        creation::{
            mixed::NewStandardOrOcmGemWell, ocm::NewOcmGemWell, standard::NewStandardGemWell,
        },
        record::ChromiumRunRecord,
    },
    id::NoId,
};

pub mod mixed;
pub mod ocm;
pub mod standard;

pub type NewChromiumRunRecord = ChromiumRunRecord<NoId>;

pub const MAX_GEM_WELLS_PER_OCM_RUN: usize = 2;
pub const MAX_GEM_WELLS_PER_NON_OCM_RUN: usize = 8;

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged, deny_unknown_fields))]
pub enum LoadedEntity {
    Suspension { suspension_id: Uuid },
    SuspensionPool { suspension_pool_id: Uuid },
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "plexy", rename_all = "snake_case"))]
pub enum NewChromiumRun {
    Mixed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChromiumRunRecord,
        gem_wells: NonemptyBoundedVec<NewStandardOrOcmGemWell, MAX_GEM_WELLS_PER_OCM_RUN>,
    },
    OnChipMultiplexing {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChromiumRunRecord,
        gem_wells: NonemptyBoundedVec<NewOcmGemWell, MAX_GEM_WELLS_PER_OCM_RUN>,
    },
    Standard {
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChromiumRunRecord,
        gem_wells: NonemptyBoundedVec<NewStandardGemWell, MAX_GEM_WELLS_PER_NON_OCM_RUN>,
    },
}
