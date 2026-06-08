use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, DefaultDesc, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(api_key_public).")]
#[strum_discriminants(
    name(ApiKeyField),
    sort_field_enum,
    strum(prefix = "(api_key_public).")
)]
pub enum ApiKeyPredicate {
    Id(UuidOperator),
    Description(StringOperator),
    PersonId(UuidOperator),
    ServiceId(UuidOperator),
    CreatedAt(TimestampOperator),
    ExpiresAt(TimestampOperator),
}

impl Default for ApiKeyField {
    fn default() -> Self {
        Self::CreatedAt
    }
}

impl DefaultDesc for ApiKeyField {}

pub type ApiKeyQuery = ComplexQuery<ApiKeyPredicate, ApiKeyField>;

pub type SimpleApiKeyQuery = SimpleQuery<ApiKeyField>;
