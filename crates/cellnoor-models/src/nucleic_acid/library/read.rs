#[cfg(feature = "app")]
use cellnoor_schema::{cdna, libraries};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{cdna::CdnaSummary, library::common::LibraryFields};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = libraries, base_query = libraries::table.inner_join(cdna::table)))]
pub struct LibrarySummary {
    id: Uuid,
    project_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LibraryFields,
    number_of_sample_index_pcr_cycles: i32,
    target_reads_per_cell: Option<i64>,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    prepared_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(embed))]
    links: LibraryLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = libraries))]
pub struct LibraryLinks {
    measurements: String,
    sequencing_runs: String,
    chromium_datasets: String,
}

impl LibrarySummary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn prepared_at(&self) -> Timestamp {
        self.prepared_at
    }

    #[must_use]
    pub fn project_id(&self) -> Uuid {
        self.project_id
    }
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

impl Library {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.summary.id()
    }

    #[must_use]
    pub fn prepared_at(&self) -> Timestamp {
        self.summary.prepared_at()
    }
}
