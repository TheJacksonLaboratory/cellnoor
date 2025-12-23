#[cfg(feature = "app")]
use cellnoor_schema::sequencing_runs;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;

#[order_by(sequencing_runs)]
#[allow(non_camel_case_types)]
pub enum SequencingRunOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    begun_at { descending: Option<bool> },
    finished_at { descending: Option<bool> },
}

impl Default for SequencingRunOrderBy {
    fn default() -> Self {
        Self::begun_at {
            descending: Some(true),
        }
    }
}

#[filter]
pub struct SequencingRunFilter {}

#[cfg(feature = "app")]
pub type SequencingRunQuery =
    crate::generic_query::Query<SequencingRunFilter, SequencingRunOrderBy>;

uuid_newtype!(SequencingRunId, "/{id}");

uuid_newtype!(SequencingRunIdLibraries, "/{id}/libraries");
