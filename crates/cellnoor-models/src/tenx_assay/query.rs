#[cfg(feature = "app")]
use cellnoor_schema::tenx_assays;
use macro_attributes::{filter, order_by};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::tenx_assay::common::{LibraryType, SampleMultiplexing};

#[filter]
pub struct TenxAssayFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
    pub library_types: Option<Vec<Vec<LibraryType>>>,
    pub sample_multiplexing: Option<Vec<SampleMultiplexing>>,
    pub chemistry_versions: Option<Vec<String>>,
    pub chromium_chips: Option<Vec<String>>,
}

#[order_by(tenx_assays)]
#[allow(non_camel_case_types)]
pub enum TenxAssayOrderBy {
    id { descending: Option<bool> },
    name { descending: Option<bool> },
    library_types { descending: Option<bool> },
    sample_multiplexing { descending: Option<bool> },
    chemistry_version { descending: Option<bool> },
    protocol_url { descending: Option<bool> },
    chromium_chip { descending: Option<bool> },
    cmdlines { descending: Option<bool> },
}

impl Default for TenxAssayOrderBy {
    fn default() -> Self {
        Self::name { descending: None }
    }
}

#[cfg(feature = "app")]
pub type TenxAssayQuery = generic_query::Query<TenxAssayFilter, TenxAssayOrderBy>;
