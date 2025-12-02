use macro_attributes::base_model;
use non_empty::NonEmptyVec;
use uuid::Uuid;

use crate::suspension::common::SuspensionFields;

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "typescript", ts(concrete(V = Option<Vec<Uuid>>)))]

pub struct SuspensionCreationInner<V> {
    #[serde(flatten)]
    pub inner: SuspensionFields,
    pub preparer_ids: NonEmptyVec<Uuid>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<Vec<Uuid>>"))]
    pub tag_ids: V,
}

#[base_model]
#[derive(serde::Deserialize)]
pub struct SuspensionCreation(pub SuspensionCreationInner<Option<Vec<Uuid>>>);
