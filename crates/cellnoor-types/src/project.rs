use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ProjectFilter, ProjectOrderBy, ProjectPredicate, ProjectQuery};
use uuid::Uuid;

use crate::simple_links::SimpleLinks;

mod query;

#[base_model]
pub struct NewProject {
    pub name: NonemptyString,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
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

#[base_model]
pub struct Project {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: ProjectRecord,
    pub links: ProjectLinks,
}
