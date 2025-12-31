#[cfg(feature = "app")]
use cellnoor_schema::labs;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[order_by(labs)]
#[allow(non_camel_case_types)]
pub enum LabOrderBy {
    id { descending: Option<bool> },
    name { descending: Option<bool> },
    delivery_dir { descending: Option<bool> },
    pi_id { descending: Option<bool> },
}

impl Default for LabOrderBy {
    fn default() -> Self {
        Self::name { descending: None }
    }
}

#[filter]
pub struct LabFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
}

#[cfg(feature = "app")]
pub type LabQuery = crate::generic_query::Query<LabFilter, LabOrderBy>;

uuid_newtype!(LabId, "/{id}");

uuid_newtype!(LabIdMembers, "/{id}/members");
