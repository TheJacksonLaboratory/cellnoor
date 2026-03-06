#[cfg(feature = "app")]
use cellnoor_schema::{chromium_runs, gem_pools};
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;

#[filter]
pub struct ChromiumRunFilter {
    pub ids: Option<Vec<Uuid>>,
    pub readable_ids: Option<Vec<String>>,
    pub assay_ids: Option<Vec<Uuid>>,
    pub project_ids: Option<Vec<Uuid>>,
    pub run_by: Option<Vec<Uuid>>,
    pub run_before: Option<Timestamp>,
    pub run_after: Option<Timestamp>,
    pub succeeded: Option<bool>,
    pub additional_data: Option<serde_json::Value>,
}

#[order_by(chromium_runs)]
#[allow(non_camel_case_types)]
pub enum ChromiumRunOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    assay_id { descending: Option<bool> },
    run_at { descending: Option<bool> },
    run_by { descending: Option<bool> },
    succeeded { descending: Option<bool> },
}

impl Default for ChromiumRunOrderBy {
    fn default() -> Self {
        Self::run_at {
            descending: Some(true),
        }
    }
}

pub type ChromiumRunQuery = generic_query::Query<ChromiumRunFilter, ChromiumRunOrderBy>;

#[filter]
pub struct GemPoolFilter {
    pub ids: Option<Vec<Uuid>>,
    pub readable_ids: Option<Vec<String>>,
    pub chromium_run_ids: Option<Vec<Uuid>>,
    pub project_ids: Option<Vec<Uuid>>,
}

#[order_by(gem_pools)]
#[allow(non_camel_case_types)]
pub enum GemPoolOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
}

impl Default for GemPoolOrderBy {
    fn default() -> Self {
        Self::id { descending: None }
    }
}

pub type GemPoolQuery = generic_query::Query<GemPoolFilter, GemPoolOrderBy>;
