use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    Relation,
    operator::{StringOperator, TimestampOperator, UuidOperator},
    project::SavedProjectRecord,
    query::{ComplexQuery, Field, OrderField, SimpleQuery},
};

#[predicate_enum(ProjectField)]
pub enum ProjectPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    CreatedBy(UuidOperator),
    #[strum(serialize = "active_period.started_at")]
    #[strum_discriminants(strum(serialize = "active_period.started_at"))]
    StartedAt(TimestampOperator),
    #[strum(serialize = "active_period.ended_at")]
    #[strum_discriminants(strum(serialize = "active_period.ended_at"))]
    EndedAt(TimestampOperator),
}

impl Field for ProjectField {
    const RELATION: &'static str = <SavedProjectRecord as Relation>::NAME;
}

impl OrderField for ProjectField {
    fn default_field() -> Self {
        Self::StartedAt
    }
}

pub type ProjectQuery = ComplexQuery<ProjectPredicate, ProjectField>;

pub type SimpleProjectQuery = SimpleQuery<ProjectField>;
