use macro_attributes::base_model;
use non_empty::NonEmptyVec;
#[cfg(feature = "app")]
use schemars::JsonSchema;
use uuid::Uuid;

use crate::chromium_run::{
    MAX_SUSPENSIONS_PER_OCM_GEM_POOL, OcmChipLoading, StandardChipLoading, common::GemPoolFields,
};

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
#[serde(untagged, rename_all = "snake_case")]
pub enum MixedChipLoading {
    Ocm(OcmChipLoading),
    Standard(StandardChipLoading),
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
pub struct MixedGemPool {
    #[serde(flatten)]
    pub inner: GemPoolFields,
    pub loading: NonEmptyVec<MixedChipLoading, MAX_SUSPENSIONS_PER_OCM_GEM_POOL>,
}

impl MixedGemPool {
    fn loading(&self) -> &[MixedChipLoading] {
        self.loading.as_ref()
    }

    #[must_use]
    pub fn suspension_ids(&self) -> Vec<Uuid> {
        self.loading()
            .iter()
            .filter_map(|l| match l {
                MixedChipLoading::Ocm(l) => l.suspension_id(),
                MixedChipLoading::Standard(l) => l.suspension_id(),
            })
            .collect()
    }

    #[must_use]
    pub fn suspension_pool_ids(&self) -> Vec<Uuid> {
        self.loading()
            .iter()
            .filter_map(|l| match l {
                MixedChipLoading::Ocm(l) => l.suspension_pool_id(),
                MixedChipLoading::Standard(l) => l.suspension_pool_id(),
            })
            .collect()
    }
}
