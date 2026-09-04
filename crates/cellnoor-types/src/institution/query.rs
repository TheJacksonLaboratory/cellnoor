use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, UuidOperator},
    query::{ComplexQuery, OrderField, SimpleQuery},
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

impl OrderField for InstitutionField {
    fn default_field() -> Self {
        Self::Name
    }

    fn default_desc() -> bool {
        true
    }
}

pub type InstitutionQuery = ComplexQuery<InstitutionPredicate, InstitutionField>;

pub type SimpleInstitutionQuery = SimpleQuery<InstitutionField>;
