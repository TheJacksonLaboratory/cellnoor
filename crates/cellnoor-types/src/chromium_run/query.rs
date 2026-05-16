use macro_attributes::{base_model, predicate_enum, sort_field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::{
    operator::{BoolOperator, StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery, filter::Filter},
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
}

#[base_model]
#[derive(strum::AsRefStr)]
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

#[cfg(feature = "postgres-types")]
impl ToPredicate for ChromiumRunPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Specimen(p) => p.to_predicate(),
            Self::TenxAssay(p) => p.to_predicate(),
            Self::ChromiumRun(field) => match field {
                ChromiumRunPredicateInner::Id(u)
                | ChromiumRunPredicateInner::AssayId(u)
                | ChromiumRunPredicateInner::RunBy(u) => u.to_predicate(),
                ChromiumRunPredicateInner::ReadableId(s) => s.to_predicate(),
                ChromiumRunPredicateInner::RunAt(t) => t.to_predicate(),
                ChromiumRunPredicateInner::Succeeded(b) => b.to_predicate(),
            },
        }
    }
}

impl Default for ChromiumRunField {
    fn default() -> Self {
        Self::RunAt
    }
}

pub type ChromiumRunQuery = ComplexQuery<ChromiumRunPredicate, ChromiumRunField>;

pub type SimpleChromiumRunQuery = SimpleQuery<ChromiumRunField>;
