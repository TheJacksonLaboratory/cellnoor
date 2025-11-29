use crate::suspension::common::SuspensionFields;
use macro_attributes::base_model;
use uuid::Uuid;

#[base_model]
#[derive(serde::Deserialize)]
pub struct CellSuspensionCreation {
    #[serde(flatten)]
    pub inner: SuspensionFields,
    pub preparer_ids: Vec<Uuid>,
}

#[base_model]
#[derive(serde::Deserialize)]
pub struct NucleusSuspensionCreation {
    #[serde(flatten)]
    pub inner: SuspensionFields,
    pub preparer_ids: Vec<Uuid>,
}
