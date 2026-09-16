use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{ApiKeyField, ApiKeyPredicate, ApiKeyQuery, SimpleApiKeyQuery};
use uuid::Uuid;

use crate::Relation;

mod query;

#[base_model]
#[derive(Default)]
pub struct NewApiKey {
    pub description: Option<NonemptyString>,
    // The person or service the key acts as. It defaults to whoever creates the key
    pub owner_id: Option<Uuid>,
    pub expires_at: Option<Timestamp>,
}

#[base_model]
pub struct ApiKeyUpdate {
    pub description: Option<NonemptyString>,
    pub expires_at: Option<Timestamp>,
}

impl Relation for ApiKeyUpdate {
    const NAME: &'static str = "api_key";
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "api_key_public"))]
pub struct SavedApiKeyRecord {
    pub id: Uuid,
    pub description: Option<NonemptyString>,
    pub owner_id: Uuid,
    pub owner_is_staff: bool,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
}

impl Relation for SavedApiKeyRecord {
    const NAME: &'static str = "api_key_public";
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
