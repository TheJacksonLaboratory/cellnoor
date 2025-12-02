use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
#[cfg(feature = "app")]
use scamplers_schema::{chromium_runs, gems};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;

#[filter]
pub struct ChromiumRunFilter {
    pub ids: Option<Vec<Uuid>>,
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
        Self::run_at { descending: None }
    }
}

#[cfg(feature = "app")]
pub type ChromiumRunQuery = generic_query::Query<ChromiumRunFilter, ChromiumRunOrderBy>;

uuid_newtype!(ChromiumRunId, "/{id}");

#[filter]
pub struct GemsFilter {
    pub ids: Option<Vec<Uuid>>,
}

#[order_by(gems)]
#[allow(non_camel_case_types)]
pub enum GemsOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
}

impl Default for GemsOrderBy {
    fn default() -> Self {
        Self::id { descending: None }
    }
}

#[cfg(feature = "app")]
pub type GemsQuery = generic_query::Query<GemsFilter, GemsOrderBy>;
