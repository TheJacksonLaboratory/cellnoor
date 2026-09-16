use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    operator::{BoolOperator, StringOperator, UuidOperator},
    person::SavedPersonRecord,
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(person_public).")]
#[strum_discriminants(name(PersonField), sort_field_enum)]
pub enum PersonPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    Email(StringOperator),
    InstitutionId(UuidOperator),
    IsStaff(BoolOperator),
    Orcid(StringOperator),
}

impl Field for PersonField {
    const RELATION: &'static str = <SavedPersonRecord as Relation>::NAME;
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
