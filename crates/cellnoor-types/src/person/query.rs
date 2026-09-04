use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{BoolOperator, StringOperator, UuidOperator},
    query::{ComplexQuery, OrderField, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(person_public).")]
#[strum_discriminants(name(PersonField), sort_field_enum, strum(prefix = "(person_public)."))]
pub enum PersonPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    Email(StringOperator),
    InstitutionId(UuidOperator),
    IsStaff(BoolOperator),
    CanManageUsers(BoolOperator),
    Orcid(StringOperator),
}

impl OrderField for PersonField {
    fn default_field() -> Self {
        Self::Name
    }

    fn default_desc() -> bool {
        false
    }
}

pub type PersonQuery = ComplexQuery<PersonPredicate, PersonField>;

pub type SimplePersonQuery = SimpleQuery<PersonField>;
