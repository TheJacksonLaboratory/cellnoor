use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ServiceField, ServicePredicate, ServiceQuery, SimpleServiceQuery};
use uuid::Uuid;

use crate::{
    id::{Id, NoId},
    person::{PermissionsToGrant, PermissionsToRevoke},
    service::record::ServiceRecord,
};

mod query;

mod record {
    use macro_attributes::select;
    use nonempty::NonemptyString;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "service"))]
    pub struct ServiceRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub description: Option<NonemptyString>,
        pub can_admin_all_projects: bool,
        pub can_admin_users: bool,
    }
}

pub type NewServiceRecord = ServiceRecord<NoId>;

#[base_model]
pub struct NewService {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewServiceRecord,
    pub users: Vec<Uuid>,
    pub permissions_to_grant: PermissionsToGrant,
}

pub type Service = ServiceRecord<Id>;

#[base_model]
pub struct ServiceUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewServiceRecord,
    pub permissions_to_grant: Option<PermissionsToGrant>,
    pub permissions_to_revoke: Option<PermissionsToRevoke>,
}
