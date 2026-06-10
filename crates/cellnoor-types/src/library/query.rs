use macro_attributes::{base_model, predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    operator::{
        I32Operator, I64Operator, JsonOperator, StringOperator, TimestampOperator, UuidOperator,
    },
    query::{ComplexQuery, DefaultDesc, SimpleQuery, filter::Filter},
    specimen::SpecimenPredicate,
};

#[predicate_enum]
#[strum(prefix = "(library).")]
#[strum_discriminants(name(LibraryField), sort_field_enum, strum(prefix = "(library)."))]
pub enum LibraryPredicateInner {
    Id(UuidOperator),
    ReadableId(StringOperator),
    CdnaId(UuidOperator),
    SingleIndexSetName(StringOperator),
    DualIndexSetName(StringOperator),
    NumberOfSampleIndexPcrCycles(I32Operator),
    TargetReadsPerCell(I64Operator),
    PreparedAt(TimestampOperator),
    AdditionalData(JsonOperator),
}

#[predicate_enum_wrapper]
pub enum LibraryPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    Library(LibraryPredicateInner),
}

impl From<SpecimenPredicate> for LibraryPredicate {
    fn from(value: SpecimenPredicate) -> Self {
        Self::Specimen(value)
    }
}

impl From<LibraryPredicateInner> for LibraryPredicate {
    fn from(value: LibraryPredicateInner) -> Self {
        Self::Library(value)
    }
}

impl From<LibraryPredicateInner> for Filter<LibraryPredicate> {
    fn from(value: LibraryPredicateInner) -> Self {
        Self::Leaf(value.into())
    }
}

impl Default for LibraryField {
    fn default() -> Self {
        Self::PreparedAt
    }
}

impl DefaultDesc for LibraryField {}

pub type LibraryQuery = ComplexQuery<LibraryPredicate, LibraryField>;

pub type SimpleLibraryQuery = SimpleQuery<LibraryField>;
