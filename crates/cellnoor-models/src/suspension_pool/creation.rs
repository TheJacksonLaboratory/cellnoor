#[cfg(feature = "app")]
use cellnoor_schema::suspension_tagging;
use macro_attributes::{base_model, insert};
use non_empty::NonEmptyVec;
use uuid::Uuid;

use crate::suspension_pool::common::SuspensionPoolFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = suspension_tagging))]
pub struct SuspensionTagging {
    suspension_id: Uuid,
    tag_id: Uuid,
}

impl SuspensionTagging {
    #[must_use]
    pub fn suspension_id(&self) -> Uuid {
        self.suspension_id
    }
}

#[base_model]
#[derive(serde::Deserialize)]
pub struct SuspensionPoolCreation {
    #[serde(flatten)]
    pub inner: SuspensionPoolFields,
    pub preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
    pub suspensions: NonEmptyVec<SuspensionTagging, { usize::MAX }>,
}
