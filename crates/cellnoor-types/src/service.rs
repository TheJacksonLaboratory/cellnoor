use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ServiceField, ServicePredicate, ServiceQuery, SimpleServiceQuery};
use uuid::Uuid;

use crate::person::{PermissionsToGrant, PermissionsToRevoke};

mod query;

#[base_model]
pub struct NewServiceRecord {
    pub description: Option<NonemptyString>,
    pub is_staff: bool,
    pub can_admin_users: bool,
}

#[base_model]
pub struct NewService {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewServiceRecord,
    pub users: Vec<Uuid>,
    pub permissions_to_grant: PermissionsToGrant,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "service"))]
pub struct Service {
    pub id: Uuid,
    pub description: Option<NonemptyString>,
    pub owned_by: Uuid,
    pub is_staff: bool,
    pub can_admin_users: bool,
    pub created_at: Timestamp,
}

#[base_model]
pub struct ServiceUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewServiceRecord,
    pub permissions_to_grant: Option<PermissionsToGrant>,
    pub permissions_to_revoke: Option<PermissionsToRevoke>,
}
