use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    institution::SavedInstitutionRecord,
    operator::{StringOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(institution).")]
#[strum_discriminants(name(InstitutionField), sort_field_enum)]
pub enum InstitutionPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    MicrosoftEntraTenantId(UuidOperator),
}

impl Field for InstitutionField {
    const RELATION: &'static str = <SavedInstitutionRecord as Relation>::NAME;
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
