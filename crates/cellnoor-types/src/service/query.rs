use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    operator::{BoolOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
    service::Service,
};

#[predicate_enum]
#[strum(prefix = "(service_public).")]
#[strum_discriminants(name(ServiceField), sort_field_enum)]
pub enum ServicePredicate {
    Id(UuidOperator),
    Description(StringOperator),
    OwnedBy(UuidOperator),
    IsStaff(BoolOperator),
    CreatedAt(TimestampOperator),
}

impl Field for ServiceField {
    const RELATION: &'static str = <Service as Relation>::NAME;
}

impl OrderField for ServiceField {
    fn default_field() -> Self {
        Self::CreatedAt
    }
}

pub type ServiceQuery = ComplexQuery<ServicePredicate, ServiceField>;

pub type SimpleServiceQuery = SimpleQuery<ServiceField>;
