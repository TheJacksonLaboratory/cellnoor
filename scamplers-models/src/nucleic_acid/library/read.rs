#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{cdna, libraries};
use uuid::Uuid;

use crate::{cdna::CdnaSummary, library::common::LibraryFields};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = libraries))]
pub struct LibrarySummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LibraryFields,
    number_of_sample_index_pcr_cycles: i32,
    target_reads_per_cell: i64,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    prepared_at: Timestamp,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = libraries, base_query = libraries::table.inner_join(cdna::table)))]
pub struct Library {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: LibrarySummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    cdna: CdnaSummary,
}
