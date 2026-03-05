#[cfg(feature = "app")]
use cellnoor_schema::suspension_tagging;
use macro_attributes::{base_model, insert};
use non_empty::NonEmptyVec;
#[cfg(feature = "app")]
use schemars::JsonSchema;
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
#[derive(serde::Deserialize, strum::IntoStaticStr)]
#[serde(tag = "multiplexing_type", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case", const_into_str)]
#[cfg_attr(feature = "app", derive(JsonSchema))]
pub enum NewSuspensionPool {
    ExogenousTag {
        #[serde(flatten)]
        inner: SuspensionPoolFields,
        preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
        suspensions: NonEmptyVec<SuspensionTagging, { usize::MAX }>,
    },
    Genetic {
        #[serde(flatten)]
        inner: SuspensionPoolFields,
        preparer_ids: NonEmptyVec<Uuid, { usize::MAX }>,
        suspensions: NonEmptyVec<Uuid, { usize::MAX }>,
    },
}

impl NewSuspensionPool {
    #[must_use]
    pub fn split_for_insertion(
        self,
    ) -> (
        SuspensionPoolFields,
        NonEmptyVec<Uuid, { usize::MAX }>,
        Option<NonEmptyVec<SuspensionTagging, { usize::MAX }>>,
        Option<NonEmptyVec<Uuid, { usize::MAX }>>,
    ) {
        match self {
            Self::ExogenousTag {
                inner,
                preparer_ids,
                suspensions,
            } => (inner, preparer_ids, Some(suspensions), None),
            Self::Genetic {
                inner,
                preparer_ids,
                suspensions,
            } => (inner, preparer_ids, None, Some(suspensions)),
        }
    }
}
