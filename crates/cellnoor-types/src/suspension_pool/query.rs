use macro_attributes::{base_model, predicate_enum, sort_field_enum};

use crate::{
    operator::{JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery, filter::Filter},
    specimen::SpecimenPredicate,
};

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
    MultiplexingType(StringOperator),
    PooledAt(TimestampOperator),
    AdditionalData(JsonOperator),
}

#[base_model]
#[derive(strum::AsRefStr)]
pub enum SuspensionPoolPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    SuspensionPool(SuspensionPoolPredicateInner),
}

impl From<SpecimenPredicate> for SuspensionPoolPredicate {
    fn from(value: SpecimenPredicate) -> Self {
        Self::Specimen(value)
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

#[cfg(feature = "postgres-types")]

impl Default for SuspensionPoolField {
    fn default() -> Self {
        Self::PooledAt
    }
}

pub type SuspensionPoolQuery = ComplexQuery<SuspensionPoolPredicate, SuspensionPoolField>;

pub type SimpleSuspensionPoolQuery = SimpleQuery<SuspensionPoolField>;
