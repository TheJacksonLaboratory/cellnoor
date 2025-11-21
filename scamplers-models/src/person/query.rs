use macro_attributes::{filter, ordinal_columns, typescript_query};
use macros::uuid_newtype;
use uuid::Uuid;

use crate::generic_query::{self};

#[ordinal_columns]
#[cfg_attr(feature = "typescript", ts(rename = "PersonOrdinalColumns"))]
pub enum OrdinalColumns {
    Id,
    Email,
    #[default]
    Name,
}

#[filter]
pub struct Filter {
    ids: Option<Vec<Uuid>>,
    names: Option<Vec<String>>,
    emails: Option<Vec<String>>,
    orcids: Option<Vec<String>>,
    microsoft_entra_oids: Option<Vec<Uuid>>,
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

    #[must_use]
    pub fn emails(&self) -> Option<&[String]> {
        self.emails.as_deref()
    }

    #[must_use]
    pub fn orcids(&self) -> Option<&[String]> {
        self.orcids.as_deref()
    }

    #[must_use]
    pub fn microsoft_entra_oids(&self) -> Option<&[Uuid]> {
        self.microsoft_entra_oids.as_deref()
    }
}

#[cfg(not(feature = "typescript"))]
pub type Query = generic_query::Query<Filter, OrdinalColumns>;

#[typescript_query]
#[cfg_attr(feature = "typescript", ts(rename = "PersonQuery"))]
pub struct Query(#[ts(inline)] generic_query::Query<Filter, OrdinalColumns>);

uuid_newtype!(PersonId, "/{id}");
