use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    Relation,
    operator::{
        F32Operator, I64Operator, JsonOperator, StringOperator, TimestampOperator, UuidOperator,
    },
    query::{ComplexQuery, Field, OrderField, SimpleQuery, filter::Operator},
    specimen::SpecimenPredicate,
    suspension::{SavedSuspensionRecord, SuspensionContent},
};

pub type SuspensionContentOperator = Operator<SuspensionContent>;

#[predicate_enum(SuspensionField)]
pub enum SuspensionPredicateInner {
    Id(UuidOperator),
    ReadableId(StringOperator),
    SpecimenId(UuidOperator),
    Content(SuspensionContentOperator),
    CreatedAt(TimestampOperator),
    LysisDurationMinutes(F32Operator),
    TargetCellRecovery(I64Operator),
    AdditionalData(JsonOperator),
}

#[predicate_enum_wrapper]
pub enum SuspensionPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    Suspension(SuspensionPredicateInner),
}

impl Field for SuspensionField {
    const RELATION: &'static str = <SavedSuspensionRecord as Relation>::NAME;
}

impl OrderField for SuspensionField {
    fn default_field() -> Self {
        Self::CreatedAt
    }
}

pub type SuspensionQuery = ComplexQuery<SuspensionPredicate, SuspensionField>;

pub type SimpleSuspensionQuery = SimpleQuery<SuspensionField>;
