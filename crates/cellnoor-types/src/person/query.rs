use macro_attributes::{predicate_enum, sort_field_enum};

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::{
    operator::{StringOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(person_public).")]
#[strum_discriminants(name(PersonField), sort_field_enum, strum(prefix = "(person_public)."))]
pub enum PersonPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    Email(StringOperator),
    InstitutionId(UuidOperator),
    Orcid(StringOperator),
}

#[cfg(feature = "postgres-types")]
impl ToPredicate for PersonPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn postgres_types::ToSql + Sync)) {
        match self {
            Self::Id(u) | Self::InstitutionId(u) => u.to_predicate(),
            Self::Name(s) | Self::Email(s) | Self::Orcid(s) => s.to_predicate(),
        }
    }
}

impl Default for PersonField {
    fn default() -> Self {
        Self::Name
    }
}

pub type PersonQuery = ComplexQuery<PersonPredicate, PersonField>;

pub type SimplePersonQuery = SimpleQuery<PersonField>;
