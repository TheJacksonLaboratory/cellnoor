use macro_attributes::{base_model, predicate_enum, sort_field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::{
    F32Operator, I64Operator, StringOperator, TimestampOperator, UuidOperator,
    query::{
        ComplexQuery, SimpleQuery,
        filter::{Filter, Operator},
    },
    specimen::SpecimenPredicate,
    suspension::SuspensionContent,
};

pub type SuspensionContentOperator = Operator<SuspensionContent>;

#[predicate_enum]
#[strum(prefix = "(suspension).")]
#[strum_discriminants(
    name(SuspensionField),
    sort_field_enum,
    strum(prefix = "(suspension).")
)]
pub enum SuspensionPredicateInner {
    Id(UuidOperator),
    ReadableId(StringOperator),
    SpecimenId(UuidOperator),
    Content(SuspensionContentOperator),
    CreatedAt(TimestampOperator),
    LysisDurationMinutes(F32Operator),
    TargetCellRecovery(I64Operator),
}

#[base_model]
#[derive(strum::AsRefStr)]
pub enum SuspensionPredicate {
    #[strum(transparent)]
    Specimen(SpecimenPredicate),
    #[cfg_attr(feature = "serde", serde(untagged))]
    #[strum(transparent)]
    Suspension(SuspensionPredicateInner),
}

impl From<SpecimenPredicate> for SuspensionPredicate {
    fn from(value: SpecimenPredicate) -> Self {
        Self::Specimen(value)
    }
}

impl From<SuspensionPredicateInner> for SuspensionPredicate {
    fn from(value: SuspensionPredicateInner) -> Self {
        Self::Suspension(value)
    }
}

impl From<SuspensionPredicateInner> for Filter<SuspensionPredicate> {
    fn from(value: SuspensionPredicateInner) -> Self {
        Self::Leaf(value.into())
    }
}

#[cfg(feature = "postgres-types")]
impl ToPredicate for SuspensionPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Specimen(p) => p.to_predicate(),
            Self::Suspension(field) => match field {
                SuspensionPredicateInner::Id(u) | SuspensionPredicateInner::SpecimenId(u) => {
                    u.to_predicate()
                }
                SuspensionPredicateInner::ReadableId(s) => s.to_predicate(),
                SuspensionPredicateInner::Content(c) => c.to_predicate(),
                SuspensionPredicateInner::CreatedAt(t) => t.to_predicate(),
                SuspensionPredicateInner::LysisDurationMinutes(f) => f.to_predicate(),
                SuspensionPredicateInner::TargetCellRecovery(i) => i.to_predicate(),
            },
        }
    }
}

impl Default for SuspensionField {
    fn default() -> Self {
        Self::CreatedAt
    }
}

pub type SuspensionQuery = ComplexQuery<SuspensionPredicate, SuspensionField>;

pub type SimpleSuspensionQuery = SimpleQuery<SuspensionField>;
