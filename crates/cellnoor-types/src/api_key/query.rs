use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    api_key::SavedApiKeyRecord,
    operator::{BoolOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
};

#[predicate_enum(ApiKeyField)]
pub enum ApiKeyPredicate {
    Id(UuidOperator),
    Description(StringOperator),
    OwnerId(UuidOperator),
    OwnerIsStaff(BoolOperator),
    CreatedAt(TimestampOperator),
    ExpiresAt(TimestampOperator),
}

impl Field for ApiKeyField {
    const RELATION: &'static str = <SavedApiKeyRecord as Relation>::NAME;
}

impl OrderField for ApiKeyField {
    fn default_field() -> Self {
        Self::CreatedAt
    }
}

pub type ApiKeyQuery = ComplexQuery<ApiKeyPredicate, ApiKeyField>;

pub type SimpleApiKeyQuery = SimpleQuery<ApiKeyField>;
