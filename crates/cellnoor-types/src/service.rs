use jiff::Timestamp;
use macro_attributes::{base_model, select};
pub use query::{ServiceField, ServicePredicate, ServiceQuery, SimpleServiceQuery};
use uuid::Uuid;

use crate::{Relation, nonempty::NonemptyString, permission::Permission};

mod query;

#[base_model]
pub struct ServiceSimpleFields {
    pub description: Option<NonemptyString>,
    pub is_staff: bool,
}

#[base_model]
pub struct NewService {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: ServiceSimpleFields,
    pub users: Vec<Uuid>,
    pub permissions_to_grant: Vec<Permission>,
}

pub type ServiceUpdate = ServiceSimpleFields;

impl Relation for ServiceSimpleFields {
    const NAME: &'static str = "service";
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "service_public"))]
pub struct Service {
    pub id: Uuid,
    pub description: Option<NonemptyString>,
    pub owned_by: Uuid,
    pub is_staff: bool,
    pub created_at: Timestamp,
}

impl Relation for Service {
    const NAME: &'static str = "service_public";
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "service_detailed"))]
pub struct ServiceDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    #[cfg_attr(feature = "postgres-types", postgres(name = "service_public"))]
    pub service: Service,
    pub permissions: Vec<Permission>,
    pub members: Vec<Uuid>,
}
