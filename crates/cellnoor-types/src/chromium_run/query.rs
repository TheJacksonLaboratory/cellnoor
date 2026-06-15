use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    operator::{BoolOperator, JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, DefaultDesc, SimpleQuery, filter::Filter},
    specimen::SpecimenPredicate,
    tenx_assay::TenxAssayPredicate,
};

#[predicate_enum]
#[strum(prefix = "(chromium_run).")]
#[strum_discriminants(
    name(ChromiumRunField),
    sort_field_enum,
    strum(prefix = "(chromium_run).")
)]
pub enum ChromiumRunPredicateInner {
    Id(UuidOperator),
    ReadableId(StringOperator),
    AssayId(UuidOperator),
    RunAt(TimestampOperator),
    RunBy(UuidOperator),
    Succeeded(BoolOperator),
    AdditionalData(JsonOperator),
}

#[predicate_enum_wrapper]
pub enum ChromiumRunPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[strum(transparent)]
    TenxAssay(TenxAssayPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    ChromiumRun(ChromiumRunPredicateInner),
}

impl From<SpecimenPredicate> for ChromiumRunPredicate {
    fn from(value: SpecimenPredicate) -> Self {
        Self::Specimen(value)
    }
}

impl From<TenxAssayPredicate> for ChromiumRunPredicate {
    fn from(value: TenxAssayPredicate) -> Self {
        Self::TenxAssay(value)
    }
}

impl From<ChromiumRunPredicateInner> for ChromiumRunPredicate {
    fn from(value: ChromiumRunPredicateInner) -> Self {
        Self::ChromiumRun(value)
    }
}

impl From<ChromiumRunPredicateInner> for Filter<ChromiumRunPredicate> {
    fn from(value: ChromiumRunPredicateInner) -> Self {
        Self::Leaf(value.into())
    }
}

impl Default for ChromiumRunField {
    fn default() -> Self {
        Self::RunAt
    }
}

impl DefaultDesc for ChromiumRunField {}

pub type ChromiumRunQuery = ComplexQuery<ChromiumRunPredicate, ChromiumRunField>;

pub type SimpleChromiumRunQuery = SimpleQuery<ChromiumRunField>;
