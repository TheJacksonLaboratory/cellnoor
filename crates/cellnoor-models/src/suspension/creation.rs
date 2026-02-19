#[cfg(feature = "app")]
use cellnoor_schema::suspensions;
use jiff::Timestamp;
use macro_attributes::{base_model, insert};
use non_empty::NonEmptyVec;
use ranged::{RangedF32, RangedU32};
#[cfg(feature = "app")]
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::suspension::common::SuspensionFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = suspensions), schemars(inline))]
pub struct NewSuspensionCommonFields {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionFields,
    target_cell_recovery: Option<RangedU32<0, { u32::MAX }>>,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::NullableTimestamp))]
    created_at: Option<Timestamp>,
}

impl NewSuspensionCommonFields {
    #[must_use]
    pub fn preparer_ids(&self) -> &[Uuid] {
        self.preparer_ids.as_ref()
    }

    #[must_use]
    pub fn parent_specimen_id(&self) -> Uuid {
        self.inner.parent_specimen_id
    }
}

#[base_model]
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "content", rename_all = "snake_case")]
pub enum NewSuspension {
    Cells(NewSuspensionCommonFields),
    Nuclei {
        #[serde(flatten)]
        common: NewSuspensionCommonFields,
        lysis_duration_minutes: RangedF32<0, { u32::MAX }>,
    },
}

impl NewSuspension {
    #[must_use]
    fn common(&self) -> &NewSuspensionCommonFields {
        match self {
            Self::Cells(s) => s,
            Self::Nuclei { common, .. } => common,
        }
    }

    #[must_use]
    pub fn preparer_ids(&self) -> &[Uuid] {
        self.common().preparer_ids.as_ref()
    }

    #[must_use]
    pub fn parent_specimen_id(&self) -> Uuid {
        self.common().parent_specimen_id()
    }

    #[must_use]
    pub fn created_at(&self) -> Option<Timestamp> {
        self.common().created_at
    }
}
