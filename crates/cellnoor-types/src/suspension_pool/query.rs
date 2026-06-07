use macro_attributes::{base_model, predicate_enum, sort_field_enum, unit_enum};

use crate::{
    operator::{JsonOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{
        ComplexQuery, SimpleQuery,
        filter::{Filter, Operator},
    },
    specimen::SpecimenPredicate,
    suspension_pool::MultiplexingTagType,
};

pub type MultiplexingTagTypeOperator = Operator<MultiplexingTagType>;

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
    MultiplexingType(MultiplexingTagTypeOperator),
    PooledAt(TimestampOperator),
    AdditionalData(JsonOperator),
}

#[base_model]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[derive(strum::IntoStaticStr)]
pub enum SuspensionPoolPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    SuspensionPool(SuspensionPoolPredicateInner),
}

impl SuspensionPoolPredicate {
    pub fn field_name(&self) -> &'static str {
        match self {
            Self::Specimen(p) => p.field_name(),
            Self::SuspensionPool(p) => p.field_name(),
        }
    }
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
