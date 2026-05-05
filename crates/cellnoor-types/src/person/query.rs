use macro_attributes::field_enum;

use crate::{
    DbQuery, Filter, StringOperator, UuidOperator,
    query::{
        filter::ToPredicate,
        order_by::{OrderDirection, OrderingField},
    },
};

#[field_enum]
pub enum PersonField<U, S> {
    Id(U),
    Name(S),
    Email(S),
    InstitutionId(U),
    Orcid(S),
}

pub type PersonPredicate = PersonField<UuidOperator, StringOperator>;

impl ToPredicate for PersonPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn postgres_types::ToSql + Sync)) {
        match self {
            Self::Id(u) | Self::InstitutionId(u) => u.to_predicate(),
            Self::Name(s) | Self::Email(s) | Self::Orcid(s) => s.to_predicate(),
        }
    }
}

pub type PersonFilter = Filter<PersonPredicate>;

pub type PersonOrderBy = PersonField<OrderDirection, OrderDirection>;

impl OrderingField for PersonOrderBy {
    fn direction(self) -> OrderDirection {
        match self {
            Self::Id(d)
            | Self::Name(d)
            | Self::Email(d)
            | Self::InstitutionId(d)
            | Self::Orcid(d) => d,
        }
    }
}

impl Default for PersonOrderBy {
    fn default() -> Self {
        Self::Name(OrderDirection::Desc)
    }
}

pub type PersonQuery = DbQuery<PersonFilter, PersonOrderBy>;
