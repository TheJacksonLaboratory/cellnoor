use macro_attributes::base_model;
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::chromium_run::creation::NewChipLoadingCommonFields;

#[base_model]
pub struct NewStandardSuspensionPoolLoadingRecord {
    pub suspension_pool_id: Uuid,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub common: NewChipLoadingCommonFields,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged, rename_all = "snake_case"))]
pub enum NewStandardChipLoading {
    Suspension {
        suspension_id: Uuid,
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChipLoadingCommonFields,
    },
    SuspensionPool {
        suspension_pool_id: Uuid,
        #[cfg_attr(feature = "serde", serde(flatten))]
        common: NewChipLoadingCommonFields,
    },
}

#[base_model]
pub struct NewStandardGemWell {
    pub readable_id: NonemptyString,
    pub loading: NewStandardChipLoading,
}
