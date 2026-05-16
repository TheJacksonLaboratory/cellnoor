use macro_attributes::base_model;
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::chromium_run::creation::NewChipLoadingCommonFields;

#[base_model]
pub struct NewStandardSuspensionLoadingRecord {
    pub suspension_id: Uuid,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub inner: NewChipLoadingCommonFields,
}

#[base_model]
pub struct NewStandardSuspensionPoolLoadingRecord {
    pub suspension_pool_id: Uuid,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub inner: NewChipLoadingCommonFields,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged, rename_all = "snake_case"))]
pub enum NewStandardChipLoading {
    Suspension(NewStandardSuspensionLoadingRecord),
    SuspensionPool(NewStandardSuspensionPoolLoadingRecord),
}

#[base_model]
pub struct NewStandardGemPool {
    pub readable_id: NonemptyString,
    pub loading: NewStandardChipLoading,
}
