use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ApiKeyField, ApiKeyPredicate, ApiKeyQuery, SimpleApiKeyQuery};
use uuid::Uuid;

use crate::person::{PermissionsToGrant, PermissionsToRevoke};

mod query;

#[base_model]
pub struct NewApiKey {
    pub description: Option<NonemptyString>,
    pub service_account_id: Option<Uuid>,
    pub expires_at: Option<Timestamp>,
    pub permissions_to_grant: PermissionsToGrant,
}

#[base_model]
pub struct ApiKeyUpdate {
    pub description: Option<NonemptyString>,
    pub expires_at: Option<Timestamp>,
    pub permissions_to_grant: Option<PermissionsToGrant>,
    pub permissions_to_revoke: Option<PermissionsToRevoke>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "api_key_public"))]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub description: Option<NonemptyString>,
    pub person_id: Option<Uuid>,
    pub service_account_id: Option<Uuid>,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ApiKey {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: ApiKeyRecord,
    pub secret: String,
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.record.fmt(f)
    }
}
