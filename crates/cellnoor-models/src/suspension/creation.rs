#[cfg(feature = "app")]
use cellnoor_schema::suspensions;
use jiff::Timestamp;
use macro_attributes::insert;
use non_empty::NonEmptyVec;
use ranged::{RangedF32, RangedU32};
use uuid::Uuid;

use crate::suspension::common::SuspensionFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = suspensions))]
pub struct SuspensionCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::NullableTimestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    created_at: Option<Timestamp>,
    target_cell_recovery: RangedU32<0, { u32::MAX }>,
    lysis_duration_minutes: Option<RangedF32<0, { u32::MAX }>>,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
}

impl SuspensionCreation {
    #[must_use]
    pub fn preparer_ids(&self) -> &[Uuid] {
        self.preparer_ids.as_ref()
    }

    #[must_use]
    pub fn parent_specimen_id(&self) -> Uuid {
        self.inner.parent_specimen_id
    }

    #[must_use]
    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }
}
