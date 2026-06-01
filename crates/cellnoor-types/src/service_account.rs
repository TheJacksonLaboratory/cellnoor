use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{
    ServiceAccountField, ServiceAccountPredicate, ServiceAccountQuery, SimpleServiceAccountQuery,
};
use uuid::Uuid;

mod query;

#[base_model]
pub struct NewServiceAccount {
    pub description: Option<NonemptyString>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "service_account"))]
pub struct ServiceAccount {
    pub id: Uuid,
    pub description: Option<NonemptyString>,
    pub owned_by: Uuid,
    pub created_at: Timestamp,
}
