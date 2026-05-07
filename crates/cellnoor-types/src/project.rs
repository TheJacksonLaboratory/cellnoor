use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ProjectFilter, ProjectPredicate, ProjectQuery, ProjectSortField};
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

#[base_model]
pub struct ProjectLinks {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SimpleLinks,
    pub specimens: String,
    pub chromium_datasets: String,
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
        links: ProjectLinks,
    },
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: ProjectRecordDetailed,
        links: ProjectLinks,
    },
}

impl ProjectLinks {
    fn from_id(id: Uuid) -> Self {
        let self_ = format!("/projects/{id}");

        Self {
            specimens: format!("{self_}/specimens"),
            chromium_datasets: format!("{self_}/chromium-datasets"),
            simple: SimpleLinks { self_ },
        }
    }
}

impl Project {
    pub fn from_record(record: ProjectRecord) -> Self {
        Self::Compact {
            links: ProjectLinks::from_id(record.id),
            record,
        }
    }

    pub fn from_detailed_record(record: ProjectRecordDetailed) -> Self {
        Self::Detailed {
            links: ProjectLinks::from_id(record.project.id),
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
