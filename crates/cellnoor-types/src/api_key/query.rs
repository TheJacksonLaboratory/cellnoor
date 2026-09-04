use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, OrderField, SimpleQuery},
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

impl OrderField for ApiKeyField {
    fn default_field() -> Self {
        Self::CreatedAt
    }
}

pub type ApiKeyQuery = ComplexQuery<ApiKeyPredicate, ApiKeyField>;

pub type SimpleApiKeyQuery = SimpleQuery<ApiKeyField>;
