use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(service_account).")]
#[strum_discriminants(
    name(ServiceAccountField),
    sort_field_enum,
    strum(prefix = "(service_account).")
)]
pub enum ServiceAccountPredicate {
    Id(UuidOperator),
    Description(StringOperator),
    OwnedBy(UuidOperator),
    CreatedAt(TimestampOperator),
}

#[cfg(feature = "postgres-types")]
impl Default for ServiceAccountField {
    fn default() -> Self {
        Self::CreatedAt
    }
}

pub type ServiceAccountQuery = ComplexQuery<ServiceAccountPredicate, ServiceAccountField>;

pub type SimpleServiceAccountQuery = SimpleQuery<ServiceAccountField>;
