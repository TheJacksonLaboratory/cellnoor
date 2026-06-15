use macro_attributes::{predicate_enum, sort_field_enum};

use crate::{
    operator::{StringOperator, TimestampOperator, UuidOperator},
    query::{ComplexQuery, DefaultDesc, SimpleQuery},
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

impl Default for ProjectField {
    fn default() -> Self {
        Self::StartedAt
    }
}

impl DefaultDesc for ProjectField {}

pub type ProjectQuery = ComplexQuery<ProjectPredicate, ProjectField>;

pub type SimpleProjectQuery = SimpleQuery<ProjectField>;
