#[cfg(feature = "app")]
use cellnoor_schema::cdna;
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::tenx_assay::LibraryType;

#[filter]
pub struct CdnaFilter {
    pub ids: Option<Vec<Uuid>>,
    pub readable_ids: Option<Vec<String>>,
    pub gem_pool_ids: Option<Vec<Uuid>>,
    pub project_ids: Option<Vec<Uuid>>,
    pub library_types: Option<Vec<LibraryType>>,
    pub prepared_before: Option<Timestamp>,
    pub prepared_after: Option<Timestamp>,
    pub n_amplification_cycles_less_than: Option<i32>,
    pub n_amplification_cycles_more_than: Option<i32>,
    pub additional_data: Option<serde_json::Value>,
}

#[order_by(cdna)]
#[allow(non_camel_case_types)]
pub enum CdnaOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    library_type { descending: Option<bool> },
    prepared_at { descending: Option<bool> },
    gem_pool_id { descending: Option<bool> },
    project_id { descending: Option<bool> },
    n_amplification_cycles { descending: Option<bool> },
}

impl Default for CdnaOrderBy {
    fn default() -> Self {
        Self::prepared_at {
            descending: Some(true),
        }
    }
}

pub type CdnaQuery = generic_query::Query<CdnaFilter, CdnaOrderBy>;
