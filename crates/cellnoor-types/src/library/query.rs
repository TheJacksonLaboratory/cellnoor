use macro_attributes::{base_model, predicate_enum, sort_field_enum};

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

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(strum::IntoStaticStr)]
pub enum LibraryPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    Library(LibraryPredicateInner),
}

impl LibraryPredicate {
    pub fn field_name(&self) -> &'static str {
        match self {
            Self::Specimen(p) => p.field_name(),
            Self::Library(p) => p.field_name(),
        }
    }
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
