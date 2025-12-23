#[cfg(feature = "app")]
use cellnoor_schema::sequencing_runs;
use jiff::Timestamp;
use macro_attributes::insert;

use crate::sequencing_run::common::SequencingRunFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_runs))]
pub struct SequencingRunCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SequencingRunFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    begun_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::NullableTimestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    finished_at: Option<Timestamp>,
}

impl SequencingRunCreation {
    #[must_use]
    pub fn begun_at(&self) -> Timestamp {
        self.begun_at
    }

    #[must_use]
    pub fn finished_at(&self) -> Option<Timestamp> {
        self.finished_at
    }
}
