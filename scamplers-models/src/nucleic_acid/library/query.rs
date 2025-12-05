use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
#[cfg(feature = "app")]
use scamplers_schema::libraries;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::tenx_assay::LibraryType;

#[filter]
pub struct LibraryFilter {
    pub ids: Option<Vec<Uuid>>,
    pub library_types: Option<Vec<LibraryType>>,
}

#[order_by(libraries)]
#[allow(non_camel_case_types)]
pub enum LibraryOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    cdna_id { descending: Option<bool> },
    single_index_set_name { descending: Option<bool> },
    dual_index_set_name { descending: Option<bool> },
    number_of_sample_index_pcr_cycles { descending: Option<bool> },
    target_reads_per_cell { descending: Option<bool> },
    prepared_at { descending: Option<bool> },
}

impl Default for LibraryOrderBy {
    fn default() -> Self {
        Self::prepared_at { descending: None }
    }
}

#[cfg(feature = "app")]
pub type LibraryQuery = generic_query::Query<LibraryFilter, LibraryOrderBy>;

uuid_newtype!(LibraryId, "/{id}");

uuid_newtype!(LibraryIdMeasurements, "/{id}/measurements");
