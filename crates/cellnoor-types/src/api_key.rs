use std::fmt::Display;

use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ApiKeyField, ApiKeyPredicate, ApiKeyQuery, SimpleApiKeyQuery};
use uuid::Uuid;

mod query;

#[base_model]
pub struct NewApiKey {
    pub description: Option<NonemptyString>,
    pub service_id: Option<ServiceId>,
    pub expires_at: Option<Timestamp>,
}

#[base_model]
pub struct ApiKeyUpdate {
    pub description: Option<NonemptyString>,
    pub expires_at: Option<Timestamp>,
}

#[select]
#[derive(Copy, Eq)]
#[cfg_attr(feature = "postgres-types", derive(postgres_types::ToSql))]
#[cfg_attr(feature = "postgres-types", postgres(transparent))]
pub struct PersonId(Uuid);

impl PersonId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}

impl Display for PersonId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[select]
#[derive(Copy, Eq)]
#[cfg_attr(feature = "postgres-types", derive(postgres_types::ToSql))]
#[cfg_attr(feature = "postgres-types", postgres(transparent))]
pub struct ServiceId(Uuid);

impl ServiceId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }
}

impl Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<ServiceId> for Uuid {
    fn from(value: ServiceId) -> Self {
        value.0
    }
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "api_key_public"))]
pub struct SavedApiKeyRecord {
    pub id: Uuid,
    pub description: Option<NonemptyString>,
    pub person_id: Option<PersonId>,
    pub service_id: Option<ServiceId>,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ApiKey {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedApiKeyRecord,
    pub secret: String,
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.record.fmt(f)
    }
}
