#[cfg(feature = "app")]
use cellnoor_schema::people;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[order_by(people)]
#[allow(non_camel_case_types)]
pub enum PersonOrderBy {
    id { descending: Option<bool> },
    name { descending: Option<bool> },
    email { descending: Option<bool> },
    email_verified { descending: Option<bool> },
    institution_id { descending: Option<bool> },
    orcid { descending: Option<bool> },
    microsoft_entra_oid { descending: Option<bool> },
}

impl Default for PersonOrderBy {
    fn default() -> Self {
        Self::name { descending: None }
    }
}

#[filter]
pub struct PersonFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
    pub emails: Option<Vec<String>>,
    pub institution_ids: Option<Vec<Uuid>>,
    pub orcids: Option<Vec<String>>,
    pub microsoft_entra_oids: Option<Vec<Uuid>>,
}

#[cfg(feature = "app")]
pub type PersonQuery = crate::generic_query::Query<PersonFilter, PersonOrderBy>;

uuid_newtype!(PersonId, "/{id}");
