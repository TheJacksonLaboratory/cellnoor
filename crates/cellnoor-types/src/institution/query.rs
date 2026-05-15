use macro_attributes::{predicate_enum, sort_field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::{
    operator::{StringOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(institution).")]
#[strum_discriminants(
    name(InstitutionField),
    sort_field_enum,
    strum(prefix = "(institution).")
)]
pub enum InstitutionPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    MicrosoftEntraTenantId(UuidOperator),
}

#[cfg(feature = "postgres-types")]
impl ToPredicate for InstitutionPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Id(u) | Self::MicrosoftEntraTenantId(u) => u.to_predicate(),
            Self::Name(s) => s.to_predicate(),
        }
    }
}

impl Default for InstitutionField {
    fn default() -> Self {
        Self::Name
    }
}

pub type InstitutionQuery = ComplexQuery<InstitutionPredicate, InstitutionField>;

pub type SimpleInstitutionQuery = SimpleQuery<InstitutionField>;
