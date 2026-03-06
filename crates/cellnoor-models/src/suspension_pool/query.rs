#[cfg(feature = "app")]
use cellnoor_schema::suspension_pools;
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;

#[filter]
pub struct SuspensionPoolFilter {
    pub ids: Option<Vec<Uuid>>,
    pub readable_ids: Option<Vec<String>>,
    pub project_ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
    pub pooled_before: Option<Timestamp>,
    pub pooled_after: Option<Timestamp>,
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

pub type SuspensionPoolQuery = generic_query::Query<SuspensionPoolFilter, SuspensionPoolOrderBy>;
