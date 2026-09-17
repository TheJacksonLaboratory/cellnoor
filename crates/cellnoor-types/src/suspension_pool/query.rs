use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    Relation,
    multiplexing_tag::MultiplexingTag,
    operator::{JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, Field, OrderField, SimpleQuery, filter::Operator},
    specimen::SpecimenPredicate,
    suspension_pool::{MultiplexingTagType, SavedSuspensionPoolRecord},
};

pub type MultiplexingTagTypeOperator = Operator<MultiplexingTagType>;

#[predicate_enum(MultiplexingTagField)]
pub enum MultiplexingTagPredicate {
    Type(MultiplexingTagTypeOperator),
}

#[predicate_enum(SuspensionPoolField)]
pub enum SuspensionPoolPredicateInner {
    Id(UuidOperator),
    ReadableId(StringOperator),
    Name(StringOperator),
    PooledAt(TimestampOperator),
    AdditionalData(JsonOperator),
}

impl Field for MultiplexingTagField {
    const RELATION: &'static str = <MultiplexingTag as Relation>::NAME;
}

#[predicate_enum_wrapper]
pub enum SuspensionPoolPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[strum(transparent)]
    MultiplexingTag(MultiplexingTagPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    SuspensionPool(SuspensionPoolPredicateInner),
}

impl Field for SuspensionPoolField {
    const RELATION: &'static str = <SavedSuspensionPoolRecord as Relation>::NAME;
}

impl OrderField for SuspensionPoolField {
    fn default_field() -> Self {
        Self::PooledAt
    }
}

pub type SuspensionPoolQuery = ComplexQuery<SuspensionPoolPredicate, SuspensionPoolField>;

pub type SimpleSuspensionPoolQuery = SimpleQuery<SuspensionPoolField>;
