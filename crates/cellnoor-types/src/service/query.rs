use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{BoolOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, OrderField, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(service).")]
#[strum_discriminants(name(ServiceField), sort_field_enum, strum(prefix = "(service)."))]
pub enum ServicePredicate {
    Id(UuidOperator),
    Description(StringOperator),
    OwnedBy(UuidOperator),
    IsStaff(BoolOperator),
    CanManageUsers(BoolOperator),
    CreatedAt(TimestampOperator),
}

impl OrderField for ServiceField {
    fn default_field() -> Self {
        Self::CreatedAt
    }
}

pub type ServiceQuery = ComplexQuery<ServicePredicate, ServiceField>;

pub type SimpleServiceQuery = SimpleQuery<ServiceField>;
