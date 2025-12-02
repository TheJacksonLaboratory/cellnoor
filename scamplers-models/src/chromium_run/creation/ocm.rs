use macro_attributes::{base_model, insert};
use non_empty::NonEmptyVec;
#[cfg(feature = "app")]
use scamplers_schema::chip_loadings;
use uuid::Uuid;

use crate::chromium_run::common::{ChipLoadingFields, GemsFields};

const MAX_SUSPENSIONS_IN_OCM_GEMS: usize = 4;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings))]
pub struct OcmChipLoading {
    suspension_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChipLoadingFields,
}

#[base_model]
#[derive(serde::Deserialize)]
pub struct OcmGems {
    #[serde(flatten)]
    pub inner: GemsFields,
    pub loading: NonEmptyVec<OcmChipLoading, MAX_SUSPENSIONS_IN_OCM_GEMS>,
}
