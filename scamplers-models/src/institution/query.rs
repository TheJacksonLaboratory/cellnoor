use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

// You might think it would be better to factor out the field definition into
// its own enum and just have a common, generic struct like:
// ```
// struct OrderBy<F> {
//     field: F
//     #[serde(default)]
//     descending: bool
// }
// ```
// where `F` is an enum of the table's columns. Writing the `QueryFragment`
// implementation is more difficult and less safe for this type of struct (see
// the `order_by` macro).
#[order_by(scamplers_schema::institutions)]
#[allow(non_camel_case_types)]
pub enum InstitutionOrderBy {
    id {
        #[serde(default)]
        descending: bool,
    },
    name {
        #[serde(default)]
        descending: bool,
    },
}

impl Default for InstitutionOrderBy {
    fn default() -> Self {
        Self::name { descending: false }
    }
}

#[filter]
pub struct InstitutionFilter {
    ids: Option<Vec<Uuid>>,
    names: Option<Vec<String>>,
}

impl InstitutionFilter {
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
pub type InstitutionQuery = crate::generic_query::Query<InstitutionFilter, InstitutionOrderBy>;

uuid_newtype!(InstitutionId, "/{id}");

uuid_newtype!(InstitutionIdMembers, "/{id}/members");
