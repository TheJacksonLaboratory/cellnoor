use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    Relation,
    library::SavedLibraryRecord,
    operator::{
        I32Operator, I64Operator, JsonOperator, StringOperator, TimestampOperator, UuidOperator,
    },
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
    specimen::SpecimenPredicate,
};

#[predicate_enum(LibraryField)]
#[cfg_attr(feature = "schemars", schemars(inline))]
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

impl Field for LibraryField {
    const RELATION: &'static str = <SavedLibraryRecord as Relation>::NAME;
}

impl OrderField for LibraryField {
    fn default_field() -> Self {
        Self::PreparedAt
    }
}

pub type LibraryQuery = ComplexQuery<LibraryPredicate, LibraryField>;

pub type SimpleLibraryQuery = SimpleQuery<LibraryField>;
