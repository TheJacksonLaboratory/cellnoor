#[cfg(feature = "app")]
use cellnoor_schema::sequencing_runs;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::sequencing_run::common::SequencingRunFields;

#[select]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_runs))]
pub struct SequencingRun {
    pub id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: SequencingRunFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    pub begun_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::NullableTimestamp))]
    pub finished_at: Option<Timestamp>,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: SequencingRunLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_runs))]
pub struct SequencingRunLinks {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "libraries")]
    pub libraries_link: String,
}

impl SequencingRun {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}
