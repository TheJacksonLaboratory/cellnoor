use macro_attributes::base_model;
use non_empty::NonEmptyVec;
use uuid::Uuid;

use crate::{suspension::SuspensionCreationInner, suspension_pool::common::SuspensionPoolFields};

#[base_model]
#[derive(serde::Deserialize)]
pub struct SuspensionPoolCreation {
    #[serde(flatten)]
    pub inner: SuspensionPoolFields,
    pub preparer_ids: NonEmptyVec<Uuid>,
    pub suspensions: NonEmptyVec<SuspensionCreationInner<NonEmptyVec<Uuid>>>,
}
