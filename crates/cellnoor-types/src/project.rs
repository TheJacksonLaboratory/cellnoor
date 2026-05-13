use macro_attributes::{base_model, select};
pub use query::{ProjectPredicate, ProjectQuery, SimpleProjectQuery};
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
    #[cfg_attr(feature = "serde", serde(default))]
    pub people: Vec<Uuid>,
}

pub type SavedProjectRecord = ProjectRecord<Id>;

// We don't particularly need a "detailed" view of a project, but this is a good
// exercise in implementing patterns we will use for libraries and Chromium
// datasets
#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "project_detailed"))]
pub struct SavedProjectDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub project: SavedProjectRecord,
    pub people: Vec<Uuid>,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Project {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedProjectRecord,
        links: SimpleLinks,
    },
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedProjectDetailed,
        links: SimpleLinks,
    },
}

impl SimpleLinks {
    fn for_project(id: Id) -> Self {
        SimpleLinks::from_str_and_id("/projects", id)
    }
}

impl Project {
    pub fn from_record(record: SavedProjectRecord) -> Self {
        Self::Compact {
            links: SimpleLinks::for_project(record.id),
            record,
        }
    }

    pub fn from_detailed_record(record: SavedProjectDetailed) -> Self {
        Self::Detailed {
            links: SimpleLinks::for_project(record.project.id),
            record,
        }
    }

    pub fn record(&self) -> &SavedProjectRecord {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed {
                record: SavedProjectDetailed { project, .. },
                ..
            } => project,
        }
    }
}
