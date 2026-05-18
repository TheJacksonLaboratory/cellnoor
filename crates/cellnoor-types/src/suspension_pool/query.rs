use macro_attributes::{base_model, predicate_enum, sort_field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
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
impl ToPredicate for SuspensionPoolPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Specimen(p) => p.to_predicate(),
            Self::SuspensionPool(field) => match field {
                SuspensionPoolPredicateInner::Id(u) => u.to_predicate(),
                SuspensionPoolPredicateInner::ReadableId(s)
                | SuspensionPoolPredicateInner::Name(s)
                | SuspensionPoolPredicateInner::MultiplexingType(s) => s.to_predicate(),
                SuspensionPoolPredicateInner::PooledAt(t) => t.to_predicate(),
                SuspensionPoolPredicateInner::AdditionalData(ad) => ad.to_predicate(),
            },
        }
    }
}

impl Default for SuspensionPoolField {
    fn default() -> Self {
        Self::PooledAt
    }
}

pub type SuspensionPoolQuery = ComplexQuery<SuspensionPoolPredicate, SuspensionPoolField>;

pub type SimpleSuspensionPoolQuery = SimpleQuery<SuspensionPoolField>;
