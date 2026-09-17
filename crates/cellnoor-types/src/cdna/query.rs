use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    Relation,
    cdna::{SavedCdnaRecord, creation::LibraryType},
    operator::{I32Operator, JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery, filter::Operator},
    specimen::SpecimenPredicate,
};

pub type LibraryTypeOperator = Operator<LibraryType>;

#[predicate_enum(CdnaField)]
pub enum CdnaPredicateInner {
    Id(UuidOperator),
    ReadableId(StringOperator),
    LibraryType(LibraryTypeOperator),
    PreparedAt(TimestampOperator),
    GemWellId(UuidOperator),
    NAmplificationCycles(I32Operator),
    AdditionalData(JsonOperator),
}

#[predicate_enum_wrapper]
pub enum CdnaPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    Cdna(CdnaPredicateInner),
}

impl Field for CdnaField {
    const RELATION: &'static str = <SavedCdnaRecord as Relation>::NAME;
}

impl OrderField for CdnaField {
    fn default_field() -> Self {
        Self::PreparedAt
    }
}

pub type CdnaQuery = ComplexQuery<CdnaPredicate, CdnaField>;

pub type SimpleCdnaQuery = SimpleQuery<CdnaField>;
