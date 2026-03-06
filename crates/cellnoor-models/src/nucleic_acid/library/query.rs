#[cfg(feature = "app")]
use cellnoor_schema::libraries;
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::tenx_assay::LibraryType;

#[filter]
pub struct LibraryFilter {
    pub ids: Option<Vec<Uuid>>,
    pub readable_ids: Option<Vec<String>>,
    pub cdna_ids: Option<Vec<Uuid>>,
    pub project_ids: Option<Vec<Uuid>>,
    pub single_index_set_names: Option<Vec<String>>,
    pub dual_index_set_names: Option<Vec<String>>,
    pub number_of_sample_index_pcr_cycles_less_than: Option<i32>,
    pub number_of_sample_index_pcr_cycles_more_than: Option<i32>,
    pub target_reads_per_cell_less_than: Option<i64>,
    pub target_reads_per_cell_more_than: Option<i64>,
    pub prepared_before: Option<Timestamp>,
    pub prepared_after: Option<Timestamp>,
    pub library_types: Option<Vec<LibraryType>>,
    pub additional_data: Option<serde_json::Value>,
}

#[order_by(libraries)]
#[allow(non_camel_case_types)]
pub enum LibraryOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    cdna_id { descending: Option<bool> },
    project_id { descending: Option<bool> },
    single_index_set_name { descending: Option<bool> },
    dual_index_set_name { descending: Option<bool> },
    number_of_sample_index_pcr_cycles { descending: Option<bool> },
    target_reads_per_cell { descending: Option<bool> },
    prepared_at { descending: Option<bool> },
}

impl Default for LibraryOrderBy {
    fn default() -> Self {
        Self::prepared_at {
            descending: Some(true),
        }
    }
}

pub type LibraryQuery = generic_query::Query<LibraryFilter, LibraryOrderBy>;
