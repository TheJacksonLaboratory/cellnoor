use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
#[cfg(feature = "app")]
use scamplers_schema::people;
use uuid::Uuid;

use crate::{generic_query::SetParentId, institution::InstitutionIdMembers};

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

impl SetParentId<InstitutionIdMembers> for PersonFilter {
    fn parent_ids_mut(&mut self) -> &mut Option<Vec<Uuid>> {
        &mut self.institution_ids
    }
}

#[cfg(feature = "app")]
pub type PersonQuery = crate::generic_query::Query<PersonFilter, PersonOrderBy>;

uuid_newtype!(PersonId, "/{id}");
