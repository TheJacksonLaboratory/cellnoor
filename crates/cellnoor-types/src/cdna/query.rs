use macro_attributes::{base_model, predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    cdna::creation::LibraryType,
    operator::{I32Operator, JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{
        ComplexQuery, DefaultDesc, SimpleQuery,
        filter::{Filter, Operator},
    },
    specimen::SpecimenPredicate,
};

pub type LibraryTypeOperator = Operator<LibraryType>;

#[predicate_enum]
#[strum(prefix = "(cdna).")]
#[strum_discriminants(name(CdnaField), sort_field_enum, strum(prefix = "(cdna)."))]
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

impl From<SpecimenPredicate> for CdnaPredicate {
    fn from(value: SpecimenPredicate) -> Self {
        Self::Specimen(value)
    }
}

impl From<CdnaPredicateInner> for CdnaPredicate {
    fn from(value: CdnaPredicateInner) -> Self {
        Self::Cdna(value)
    }
}

impl From<CdnaPredicateInner> for Filter<CdnaPredicate> {
    fn from(value: CdnaPredicateInner) -> Self {
        Self::Leaf(value.into())
    }
}

impl Default for CdnaField {
    fn default() -> Self {
        Self::PreparedAt
    }
}

impl DefaultDesc for CdnaField {}

pub type CdnaQuery = ComplexQuery<CdnaPredicate, CdnaField>;

pub type SimpleCdnaQuery = SimpleQuery<CdnaField>;
