use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, OrderField, SimpleQuery},
};

#[predicate_enum]
#[strum(prefix = "(project).")]
#[strum_discriminants(name(ProjectField), sort_field_enum, strum(prefix = "(project)."))]
pub enum ProjectPredicate {
    Id(UuidOperator),
    Name(StringOperator),
    CreatedByPerson(UuidOperator),
    CreatedByService(UuidOperator),
    StartedAt(TimestampOperator),
    EndedAt(TimestampOperator),
}

impl OrderField for ProjectField {
    fn default_field() -> Self {
        Self::StartedAt
    }
}

pub type ProjectQuery = ComplexQuery<ProjectPredicate, ProjectField>;

pub type SimpleProjectQuery = SimpleQuery<ProjectField>;
