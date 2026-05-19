use macro_attributes::{predicate_enum, sort_field_enum};

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

impl Default for InstitutionField {
    fn default() -> Self {
        Self::Name
    }
}

pub type InstitutionQuery = ComplexQuery<InstitutionPredicate, InstitutionField>;

pub type SimpleInstitutionQuery = SimpleQuery<InstitutionField>;
