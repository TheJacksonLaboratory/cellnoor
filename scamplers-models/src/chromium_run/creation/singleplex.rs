use macro_attributes::{base_model, insert};
#[cfg(feature = "app")]
use scamplers_schema::chip_loadings;
use uuid::Uuid;

use crate::chromium_run::common::{ChipLoadingFields, GemsFields};

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings))]
pub struct SingleplexChipLoading {
    suspension_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChipLoadingFields,
}

#[base_model]
#[derive(serde::Deserialize)]
pub struct SingleplexGems {
    #[serde(flatten)]
    pub inner: GemsFields,
    pub loading: SingleplexChipLoading,
}
