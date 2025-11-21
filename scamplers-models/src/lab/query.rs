use macro_attributes::{filter, ordinal_column, typescript_query};
use macros::uuid_newtype;
use uuid::Uuid;

use crate::generic_query::{self};

#[ordinal_column]
#[cfg_attr(feature = "typescript", ts(rename = "LabOrdinalColumn"))]
pub enum OrdinalColumn {
    Id,
    #[default]
    Name,
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

#[cfg(not(feature = "typescript"))]
pub type Query = generic_query::Query<Filter, OrdinalColumn>;

#[typescript_query]
#[cfg_attr(feature = "typescript", ts(rename = "LabQuery"))]
pub struct Query(#[ts(inline)] generic_query::Query<Filter, OrdinalColumn>);

uuid_newtype!(Id, "/{id}");
