use jiff::Timestamp;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::sequencing_runs;

use crate::sequencing_run::common::SequencingRunFields;

#[select]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_runs))]
pub struct SequencingRun {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SequencingRunFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    begun_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::NullableTimestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    finished_at: Option<Timestamp>,
}
