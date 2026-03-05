#[cfg(feature = "app")]
use cellnoor_schema::chip_loadings;
use macro_attributes::{base_model, insert};
#[cfg(feature = "app")]
use schemars::JsonSchema;
use uuid::Uuid;

use crate::chromium_run::common::{ChipLoadingFields, GemPoolFields};

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings))]
pub struct StandardChipLoading {
    suspension_id: Option<Uuid>,
    suspension_pool_id: Option<Uuid>,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChipLoadingFields,
}

impl StandardChipLoading {
    #[must_use]
    pub fn suspension_id(&self) -> Option<Uuid> {
        self.suspension_id
    }

    #[must_use]
    pub fn suspension_pool_id(&self) -> Option<Uuid> {
        self.suspension_pool_id
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
pub struct StandardGemPool {
    #[serde(flatten)]
    pub inner: GemPoolFields,
    pub loading: StandardChipLoading,
}
