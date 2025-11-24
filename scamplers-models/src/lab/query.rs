use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[order_by(scamplers_schema::labs)]
#[allow(non_camel_case_types)]
pub enum LabOrderBy {
    id {
        #[serde(default)]
        descending: bool,
    },
    name {
        #[serde(default)]
        descending: bool,
    },
}

impl Default for LabOrderBy {
    fn default() -> Self {
        Self::name { descending: false }
    }
}

#[filter]
pub struct LabFilter {
    ids: Option<Vec<Uuid>>,
    names: Option<Vec<String>>,
}

impl LabFilter {
    #[must_use]
    pub fn ids(&self) -> Option<&[Uuid]> {
        self.ids.as_deref()
    }

    #[must_use]
    pub fn names(&self) -> Option<&[String]> {
        self.names.as_deref()
    }
}

#[cfg(feature = "app")]
pub type LabQuery = crate::generic_query::Query<LabFilter, LabOrderBy>;

uuid_newtype!(LabId, "/{id}");

uuid_newtype!(LabIdMembers, "/{id}/members");
