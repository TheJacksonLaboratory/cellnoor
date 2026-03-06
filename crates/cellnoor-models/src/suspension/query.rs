#[cfg(feature = "app")]
use cellnoor_schema::suspensions;
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::suspension::SuspensionContent;

#[filter]
pub struct SuspensionFilter {
    pub ids: Option<Vec<Uuid>>,
    pub readable_ids: Option<Vec<String>>,
    pub parent_specimen_ids: Option<Vec<Uuid>>,
    pub project_ids: Option<Vec<Uuid>>,
    pub contents: Option<Vec<SuspensionContent>>,
    pub created_before: Option<Timestamp>,
    pub created_after: Option<Timestamp>,
    pub lysis_duration_less_than: Option<f32>,
    pub lysis_duration_more_than: Option<f32>,
    pub target_cell_recovery_less_than: Option<i64>,
    pub target_cell_recovery_more_than: Option<i64>,
    pub additional_data: Option<serde_json::Value>,
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

pub type SuspensionQuery = generic_query::Query<SuspensionFilter, SuspensionOrderBy>;
