use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    institution::SavedInstitutionRecord,
    operator::{StringOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
};

#[predicate_enum(InstitutionField)]
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
}

pub type InstitutionQuery = ComplexQuery<InstitutionPredicate, InstitutionField>;

pub type SimpleInstitutionQuery = SimpleQuery<InstitutionField>;
