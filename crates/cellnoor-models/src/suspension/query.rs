#[cfg(feature = "app")]
use cellnoor_schema::suspensions;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;

#[filter]
pub struct SuspensionFilter {
    pub ids: Option<Vec<Uuid>>,
}

#[order_by(suspensions)]
#[allow(non_camel_case_types)]
pub enum SuspensionOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    parent_specimen_id { descending: Option<bool> },
    created_at { descending: Option<bool> },
    lysis_duration_minutes { descending: Option<bool> },
    target_cell_recovery { descending: Option<bool> },
}

impl Default for SuspensionOrderBy {
    fn default() -> Self {
        Self::created_at {
            descending: Some(true),
        }
    }
}

#[cfg(feature = "app")]
pub type SuspensionQuery = generic_query::Query<SuspensionFilter, SuspensionOrderBy>;

uuid_newtype!(SuspensionId, "/{id}");

uuid_newtype!(SuspensionIdMeasurements, "/{id}/measurements");
