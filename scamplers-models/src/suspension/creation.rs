use macro_attributes::base_model;
use uuid::Uuid;

use crate::suspension::common::SuspensionFields;

#[base_model]
pub struct SuspensionCreation {
    #[serde(flatten)]
    inner: SuspensionFields,
    preparer_ids: Vec<Uuid>,
}

impl SuspensionCreation {
    #[must_use]
    pub fn split_for_insertion(self) -> (SuspensionFields, Vec<Uuid>) {
        let Self {
            inner,
            preparer_ids,
        } = self;

        (inner, preparer_ids)
    }
}
