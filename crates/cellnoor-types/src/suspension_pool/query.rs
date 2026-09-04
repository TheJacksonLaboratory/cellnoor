use macro_attributes::{predicate_enum, predicate_enum_wrapper, sort_field_enum};

use crate::{
    operator::{JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{
        ComplexQuery, OrderField, SimpleQuery,
        filter::{Filter, Operator},
    },
    specimen::SpecimenPredicate,
    suspension_pool::MultiplexingTagType,
};

pub type MultiplexingTagTypeOperator = Operator<MultiplexingTagType>;

#[predicate_enum]
#[strum(prefix = "(multiplexing_tag).")]
#[strum_discriminants(
    name(MultiplexingTagField),
    sort_field_enum,
    strum(prefix = "(multiplexing_tag).")
)]
pub enum MultiplexingTagPredicate {
    Type(MultiplexingTagTypeOperator),
}

#[predicate_enum]
#[strum(prefix = "(suspension_pool).")]
#[strum_discriminants(
    name(SuspensionPoolField),
    sort_field_enum,
    strum(prefix = "(suspension_pool).")
)]
pub enum SuspensionPoolPredicateInner {
    Id(UuidOperator),
    ReadableId(StringOperator),
    Name(StringOperator),
    PooledAt(TimestampOperator),
    AdditionalData(JsonOperator),
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

impl From<SpecimenPredicate> for SuspensionPoolPredicate {
    fn from(value: SpecimenPredicate) -> Self {
        Self::Specimen(value)
    }
}

impl From<MultiplexingTagPredicate> for SuspensionPoolPredicate {
    fn from(value: MultiplexingTagPredicate) -> Self {
        Self::MultiplexingTag(value)
    }
}

impl From<SuspensionPoolPredicateInner> for SuspensionPoolPredicate {
    fn from(value: SuspensionPoolPredicateInner) -> Self {
        Self::SuspensionPool(value)
    }
}

impl From<SuspensionPoolPredicateInner> for Filter<SuspensionPoolPredicate> {
    fn from(value: SuspensionPoolPredicateInner) -> Self {
        Self::Leaf(value.into())
    }
}

impl OrderField for SuspensionPoolField {
    fn default_field() -> Self {
        Self::PooledAt
    }
}

pub type SuspensionPoolQuery = ComplexQuery<SuspensionPoolPredicate, SuspensionPoolField>;

pub type SimpleSuspensionPoolQuery = SimpleQuery<SuspensionPoolField>;
