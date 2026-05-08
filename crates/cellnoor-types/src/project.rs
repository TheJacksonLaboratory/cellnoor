use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ProjectPredicate, ProjectQuery, SimpleProjectQuery};
use uuid::Uuid;

use crate::simple_links::SimpleLinks;

mod query;

#[base_model]
pub struct NewProject {
    pub name: NonemptyString,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    #[cfg_attr(feature = "serde", serde(default))]
    pub people: Vec<Uuid>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "project"))]
pub struct ProjectRecord {
    pub id: Uuid,
    pub name: NonemptyString,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
}

// We don't particularly need a "detailed" view of a project, but this is a good
// exercise in implementing patterns we will use for libraries and Chromium
// datasets
#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "project_detailed"))]
pub struct ProjectRecordDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub project: ProjectRecord,
    pub people: Vec<Uuid>,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Project {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: ProjectRecord,
        links: SimpleLinks,
    },
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: ProjectRecordDetailed,
        links: SimpleLinks,
    },
}

impl SimpleLinks {
    fn for_project(id: Uuid) -> Self {
        SimpleLinks::from_str_and_id("/projects", id)
    }
}

impl Project {
    pub fn from_record(record: ProjectRecord) -> Self {
        Self::Compact {
            links: SimpleLinks::for_project(record.id),
            record,
        }
    }

    pub fn from_detailed_record(record: ProjectRecordDetailed) -> Self {
        Self::Detailed {
            links: SimpleLinks::for_project(record.project.id),
            record,
        }
    }

    pub fn record(&self) -> &ProjectRecord {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed {
                record: ProjectRecordDetailed { project, .. },
                ..
            } => project,
        }
    }
}
