#[cfg(feature = "app")]
use cellnoor_schema::suspension_pools;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;

#[filter]
pub struct SuspensionPoolFilter {
    pub ids: Option<Vec<Uuid>>,
}

#[order_by(suspension_pools)]
#[allow(non_camel_case_types)]
pub enum SuspensionPoolOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    name { descending: Option<bool> },
    pooled_at { descending: Option<bool> },
}

impl Default for SuspensionPoolOrderBy {
    fn default() -> Self {
        Self::pooled_at {
            descending: Some(true),
        }
    }
}

#[cfg(feature = "app")]
pub type SuspensionPoolQuery = generic_query::Query<SuspensionPoolFilter, SuspensionPoolOrderBy>;

uuid_newtype!(SuspensionPoolId, "/{id}");

uuid_newtype!(SuspensionPoolIdMeasurements, "/{id}/measurements");

uuid_newtype!(SuspensionPoolIdSuspensions, "/{id}/suspensions");
