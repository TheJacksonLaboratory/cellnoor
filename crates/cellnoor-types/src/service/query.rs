use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{BoolOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(service).")]
#[strum_discriminants(name(ServiceField), sort_field_enum, strum(prefix = "(service)."))]
pub enum ServicePredicate {
    Id(UuidOperator),
    Description(StringOperator),
    OwnedBy(UuidOperator),
    CanAdminAllProjects(BoolOperator),
    CanAdminUsers(BoolOperator),
    CreatedAt(TimestampOperator),
}

#[cfg(feature = "postgres-types")]
impl Default for ServiceField {
    fn default() -> Self {
        Self::CreatedAt
    }
}

pub type ServiceQuery = ComplexQuery<ServicePredicate, ServiceField>;

pub type SimpleServiceQuery = SimpleQuery<ServiceField>;
