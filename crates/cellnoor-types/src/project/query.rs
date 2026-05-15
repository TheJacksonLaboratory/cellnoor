use macro_attributes::{predicate_enum, sort_field_enum};
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::{
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(project).")]
#[strum_discriminants(name(ProjectField), sort_field_enum, strum(prefix = "(project)."))]
pub enum ProjectPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    StartedAt(TimestampOperator),
    EndedAt(TimestampOperator),
}

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

impl Default for ProjectField {
    fn default() -> Self {
        Self::StartedAt
    }
}

pub type ProjectQuery = ComplexQuery<ProjectPredicate, ProjectField>;

pub type SimpleProjectQuery = SimpleQuery<ProjectField>;
