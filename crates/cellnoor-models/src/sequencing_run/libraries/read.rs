#[cfg(feature = "app")]
use cellnoor_schema::{libraries, sequencing_submissions};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;

use crate::library::LibrarySummary;

#[select]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_submissions, base_query = sequencing_submissions::table.inner_join(libraries::table)))]
pub struct SequencedLibrary {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: LibrarySummary,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    submitted_at: Timestamp,
}
