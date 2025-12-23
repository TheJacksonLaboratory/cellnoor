#[cfg(feature = "app")]
use cellnoor_schema::chip_loadings;
use macro_attributes::{base_model, insert};
use uuid::Uuid;

use crate::chromium_run::common::{ChipLoadingFields, GemPoolFields};

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings))]
pub struct PoolMultiplexChipLoading {
    suspension_pool_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChipLoadingFields,
}

impl PoolMultiplexChipLoading {
    #[must_use]
    pub fn suspension_pool_id(&self) -> Uuid {
        self.suspension_pool_id
    }
}

#[base_model]
#[derive(serde::Deserialize)]
pub struct PoolMultiplexGemPool {
    #[serde(flatten)]
    pub inner: GemPoolFields,
    pub loading: PoolMultiplexChipLoading,
}
