use macro_attributes::{base_model, select};
pub use query::{ProjectField, ProjectPredicate, ProjectQuery, SimpleProjectQuery};
use uuid::Uuid;

use crate::{
    id::{Id, NoId},
    project::record::ProjectRecord,
    simple_links::SimpleLinks,
};

mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use nonempty::NonemptyString;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "project"))]
    pub struct ProjectRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub name: NonemptyString,
        pub started_at: Timestamp,
        pub ended_at: Timestamp,
    }
}

pub type NewProjectRecord = ProjectRecord<NoId>;

#[base_model]
pub struct NewProject {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewProjectRecord,
    pub people: Vec<Uuid>,
}

pub type SavedProjectRecord = ProjectRecord<Id>;

// We don't particularly need a "detailed" view of a project, but this is a good
// exercise in implementing patterns we will use for libraries and Chromium
// datasets
#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "project_detailed"))]
pub struct SavedProjectRecordDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub project: SavedProjectRecord,
    pub people: Vec<Uuid>,
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
