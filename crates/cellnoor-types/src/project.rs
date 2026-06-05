use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ProjectField, ProjectPredicate, ProjectQuery, SimpleProjectQuery};
use uuid::Uuid;

use crate::simple_links::SimpleLinks;

mod query;

#[base_model]
pub struct NewProject {
    pub name: NonemptyString,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub members: Vec<Uuid>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "project"))]
pub struct SavedProjectRecord {
    pub id: Uuid,
    pub name: NonemptyString,
    pub created_by: Uuid,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
}

// We don't particularly need a "detailed" view of a project, but this is a good
// exercise in implementing patterns we will use for libraries and Chromium
// datasets
#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "project_detailed"))]
pub struct SavedProjectRecordDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub project: SavedProjectRecord,
    pub members: Vec<Uuid>,
}

#[base_model]
pub struct ProjectCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedProjectRecord,
    pub links: SimpleLinks,
}

#[base_model]
pub struct ProjectDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedProjectRecordDetailed,
    pub links: SimpleLinks,
}

impl ProjectDetailed {
    #[must_use]
    pub fn record(&self) -> &SavedProjectRecord {
        &self.record.project
    }
}
