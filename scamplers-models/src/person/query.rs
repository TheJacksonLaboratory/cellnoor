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
    name: Option<String>,
    email: Option<String>,
    orcid: Option<String>,
    microsoft_entra_oids: Option<Vec<Uuid>>,
}

impl Filter {
    #[must_use]
    pub fn ids(&self) -> Option<&[Uuid]> {
        self.ids.as_deref()
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    #[must_use]
    pub fn orcid(&self) -> Option<&str> {
        self.orcid.as_deref()
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
