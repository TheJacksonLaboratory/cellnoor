#[cfg(feature = "app")]
use cellnoor_schema::projects;
use macro_attributes::select;
use uuid::Uuid;

use crate::project::common::ProjectFields;

#[select]
#[cfg_attr(feature = "app", diesel(table_name = projects))]
pub struct Project {
    pub id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: ProjectFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    pub started_at: jiff::Timestamp,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    pub ended_at: jiff::Timestamp,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: ProjectLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = projects))]
pub struct ProjectLinks {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "people")]
    pub people_link: String,
    #[serde(rename = "specimens")]
    pub specimens_link: String,
    #[serde(rename = "chromium_datasets")]
    pub chromium_datasets_link: String,
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
