use macro_attributes::field_enum;
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::query::{
    ComplexQuery,
    filter::{Filter, StringOperator, TimestampOperator, UuidOperator},
    order_by::{OrderDirection, OrderingField},
};

#[field_enum]
#[strum(prefix = "(project).")]
pub enum ProjectField<U, S, T> {
    Id(U),
    Name(S),
    StartedAt(T),
    EndedAt(T),
}

pub type ProjectPredicate = ProjectField<UuidOperator, StringOperator, TimestampOperator>;

#[cfg(feature = "postgres-types")]
impl ToPredicate for ProjectPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Id(u) => u.to_predicate(),
            Self::Name(s) => s.to_predicate(),
            Self::StartedAt(t) | Self::EndedAt(t) => t.to_predicate(),
        }
    }
}

pub type ProjectFilter = Filter<ProjectPredicate>;

pub type ProjectSortField = ProjectField<OrderDirection, OrderDirection, OrderDirection>;

impl OrderingField for ProjectSortField {
    fn direction(self) -> OrderDirection {
        match self {
            Self::Id(d) | Self::Name(d) | Self::StartedAt(d) | Self::EndedAt(d) => d,
        }
    }
}

impl Default for ProjectSortField {
    fn default() -> Self {
        Self::StartedAt(OrderDirection::Desc)
    }
}

pub type ProjectQuery = ComplexQuery<ProjectFilter, ProjectSortField>;
