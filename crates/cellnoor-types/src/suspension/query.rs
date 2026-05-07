use macro_attributes::{base_model, field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::{
    query::{
        ComplexQuery,
        filter::{
            F32Operator, Filter, I64Operator, ScalarOperator, StringOperator, TimestampOperator,
            UuidOperator,
        },
        order_by::{OrderDirection, OrderingField},
    },
    specimen::SpecimenPredicate,
    suspension::SuspensionContent,
};

pub type SuspensionContentOperator = ScalarOperator<SuspensionContent>;

#[field_enum]
#[strum(prefix = "(suspension).")]
pub enum SuspensionField<U, S, T, C, F, I> {
    Id(U),
    ReadableId(S),
    SpecimenId(U),
    Content(C),
    CreatedAt(T),
    LysisDurationMinutes(F),
    TargetCellRecovery(I),
}

pub type SuspensionPredicateInner = SuspensionField<
    UuidOperator,
    StringOperator,
    TimestampOperator,
    SuspensionContentOperator,
    F32Operator,
    I64Operator,
>;

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

#[cfg(feature = "postgres-types")]
impl ToPredicate for SuspensionPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Specimen(p) => p.to_predicate(),
            Self::Suspension(field) => match field {
                SuspensionField::Id(u) | SuspensionField::SpecimenId(u) => u.to_predicate(),
                SuspensionField::ReadableId(s) => s.to_predicate(),
                SuspensionField::Content(c) => c.to_predicate(),
                SuspensionField::CreatedAt(t) => t.to_predicate(),
                SuspensionField::LysisDurationMinutes(f) => f.to_predicate(),
                SuspensionField::TargetCellRecovery(i) => i.to_predicate(),
            },
        }
    }
}

pub type SuspensionFilter = Filter<SuspensionPredicate>;

pub type SuspensionSortField = SuspensionField<
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
    OrderDirection,
>;

impl OrderingField for SuspensionSortField {
    fn direction(self) -> OrderDirection {
        match self {
            Self::Id(d)
            | Self::ReadableId(d)
            | Self::SpecimenId(d)
            | Self::Content(d)
            | Self::CreatedAt(d)
            | Self::LysisDurationMinutes(d)
            | Self::TargetCellRecovery(d) => d,
        }
    }
}

impl Default for SuspensionSortField {
    fn default() -> Self {
        Self::CreatedAt(OrderDirection::Desc)
    }
}

pub type SuspensionQuery = ComplexQuery<SuspensionFilter, SuspensionSortField>;
