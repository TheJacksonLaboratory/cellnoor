#[cfg(feature = "app")]
use cellnoor_schema::projects;
use macro_attributes::select;
use uuid::Uuid;

use crate::{links::Links, project::common::ProjectFields};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = projects))]
pub struct Project {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ProjectFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    started_at: jiff::Timestamp,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    ended_at: jiff::Timestamp,
    links: Links,
}
impl Project {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name.as_ref()
    }
}
