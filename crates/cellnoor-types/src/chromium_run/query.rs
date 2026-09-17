use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    Relation,
    chromium_run::SavedChromiumRunRecord,
    operator::{BoolOperator, JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
    specimen::SpecimenPredicate,
    tenx_assay::TenxAssayPredicate,
};

#[predicate_enum(ChromiumRunField)]
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

impl Field for ChromiumRunField {
    const RELATION: &'static str = <SavedChromiumRunRecord as Relation>::NAME;
}

impl OrderField for ChromiumRunField {
    fn default_field() -> Self {
        Self::RunAt
    }
}

pub type ChromiumRunQuery = ComplexQuery<ChromiumRunPredicate, ChromiumRunField>;

pub type SimpleChromiumRunQuery = SimpleQuery<ChromiumRunField>;
