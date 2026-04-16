#[cfg(feature = "app")]
use cellnoor_schema::projects;
use macro_attributes::insert;
use non_empty::NonEmptyString;

use crate::project::common::ProjectFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = projects))]
pub struct NewProject {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: ProjectFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    pub started_at: jiff::Timestamp,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    pub ended_at: jiff::Timestamp,
}
impl NewProject {
    #[must_use]
    pub fn new(
        name: NonEmptyString,
        (started_at, ended_at): (jiff::Timestamp, jiff::Timestamp),
    ) -> Self {
        Self {
            inner: ProjectFields { name },
            started_at,
            ended_at,
        }
    }
}
