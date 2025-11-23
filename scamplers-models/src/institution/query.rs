use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[order_by(scamplers_schema::institutions)]
#[allow(non_camel_case_types)]
pub enum OrderBy {
    id {
        #[serde(default)]
        descending: bool,
    },
    name {
        #[serde(default)]
        descending: bool,
    },
}

impl Default for OrderBy {
    fn default() -> Self {
        Self::name { descending: false }
    }
}

#[filter]
pub struct Filter {
    ids: Option<Vec<Uuid>>,
    names: Option<Vec<String>>,
}

impl Filter {
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
pub type Query = crate::generic_query::Query<Filter, OrderBy>;

uuid_newtype!(Id, "/{id}");

uuid_newtype!(IdMembers, "/{id}/members");
