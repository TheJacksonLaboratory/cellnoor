#[cfg(feature = "app")]
use cellnoor_schema::chip_loadings;
use macro_attributes::{base_model, insert};
#[cfg(feature = "app")]
use schemars::JsonSchema;
use uuid::Uuid;

use crate::chromium_run::common::{ChipLoadingFields, GemPoolFields};

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings), schemars(inline))]
pub struct SuspensionLoading {
    suspension_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChipLoadingFields,
}

impl SuspensionLoading {
    #[must_use]
    pub fn suspension_id(&self) -> Uuid {
        self.suspension_id
    }
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chip_loadings), schemars(inline))]
pub struct SuspensionPoolLoading {
    suspension_pool_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChipLoadingFields,
}

impl SuspensionPoolLoading {
    #[must_use]
    pub fn suspension_pool_id(&self) -> Uuid {
        self.suspension_pool_id
    }
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
#[serde(untagged, rename_all = "snake_case")]
pub enum StandardChipLoading {
    Suspension(SuspensionLoading),
    SuspensionPool(SuspensionPoolLoading),
}

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
pub struct StandardGemPool {
    #[serde(flatten)]
    pub inner: GemPoolFields,
    pub loading: StandardChipLoading,
}

impl StandardGemPool {
    #[must_use]
    pub fn suspension_id(&self) -> Option<Uuid> {
        match &self.loading {
            StandardChipLoading::Suspension(s) => Some(s.suspension_id()),
            StandardChipLoading::SuspensionPool(_) => None,
        }
    }

    #[must_use]
    pub fn suspension_pool_id(&self) -> Option<Uuid> {
        match &self.loading {
            StandardChipLoading::SuspensionPool(p) => Some(p.suspension_pool_id()),
            StandardChipLoading::Suspension(_) => None,
        }
    }
}
