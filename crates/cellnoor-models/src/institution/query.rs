#[cfg(feature = "app")]
use cellnoor_schema::institutions;
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
#[order_by(institutions)]
#[allow(non_camel_case_types)]
pub enum InstitutionOrderBy {
    id { descending: Option<bool> },
    name { descending: Option<bool> },
}

impl Default for InstitutionOrderBy {
    fn default() -> Self {
        Self::name { descending: None }
    }
}

#[filter]
pub struct InstitutionFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
}

#[cfg(feature = "app")]
pub type InstitutionQuery = crate::generic_query::Query<InstitutionFilter, InstitutionOrderBy>;

uuid_newtype!(InstitutionId, "/{id}");

uuid_newtype!(InstitutionIdMembers, "/{id}/members");
