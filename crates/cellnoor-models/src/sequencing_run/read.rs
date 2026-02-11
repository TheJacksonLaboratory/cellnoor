#[cfg(feature = "app")]
use cellnoor_schema::sequencing_runs;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::sequencing_run::common::SequencingRunFields;

#[select]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_runs))]
pub struct SequencingRun {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SequencingRunFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    begun_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::NullableTimestamp))]
    finished_at: Option<Timestamp>,
}

impl SequencingRun {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}
