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
pub struct SuspensionCreationCommonFields {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionFields,
    target_cell_recovery: Option<RangedU32<0, { u32::MAX }>>,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
}

impl SuspensionCreationCommonFields {
    #[must_use]
    pub fn preparer_ids(&self) -> &[Uuid] {
        self.preparer_ids.as_ref()
    }

    #[must_use]
    pub fn parent_specimen_id(&self) -> Uuid {
        self.inner.parent_specimen_id
    }
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = suspensions))]
pub struct CellSuspensionCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    common: SuspensionCreationCommonFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::NullableTimestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    created_at: Option<Timestamp>,
}

impl CellSuspensionCreation {
    #[must_use]
    pub fn common(&self) -> &SuspensionCreationCommonFields {
        &self.common
    }

    #[must_use]
    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = suspensions))]
pub struct NucleusSuspensionCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    common: SuspensionCreationCommonFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::NullableTimestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    created_at: Option<Timestamp>,
    lysis_duration_minutes: RangedF32<0, { u32::MAX }>,
}

impl NucleusSuspensionCreation {
    #[must_use]
    pub fn common(&self) -> &SuspensionCreationCommonFields {
        &self.common
    }

    #[must_use]
    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }
}
