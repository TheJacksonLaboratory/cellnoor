#[cfg(feature = "app")]
use cellnoor_schema::projects;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[order_by(projects)]
#[allow(non_camel_case_types)]
pub enum ProjectOrderBy {
    id { descending: Option<bool> },
    name { descending: Option<bool> },
    started_at { descending: Option<bool> },
    ended_at { descending: Option<bool> },
}

impl Default for ProjectOrderBy {
    fn default() -> Self {
        Self::name { descending: None }
    }
}

#[filter]
pub struct ProjectFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub started_before: Option<jiff::Timestamp>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub started_after: Option<jiff::Timestamp>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub ended_before: Option<jiff::Timestamp>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub ended_after: Option<jiff::Timestamp>,
}

#[cfg(feature = "app")]
pub type ProjectQuery = crate::generic_query::Query<ProjectFilter, ProjectOrderBy>;

uuid_newtype!(ProjectId, "/{id}");

uuid_newtype!(ProjectPeople, "/{id}/people");
