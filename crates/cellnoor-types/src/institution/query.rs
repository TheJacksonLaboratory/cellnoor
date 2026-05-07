use macro_attributes::field_enum;
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::filter::ToPredicate;
use crate::query::{
    ComplexQuery,
    filter::{Filter, StringOperator, UuidOperator},
    order_by::{OrderDirection, OrderingField},
};

#[field_enum]
#[strum(prefix = "(institution).")]
pub enum InstitutionField<U, S> {
    Id(U),
    Name(S),
    MicrosoftEntraTenantId(U),
}

pub type InstitutionPredicate = InstitutionField<UuidOperator, StringOperator>;

#[cfg(feature = "postgres-types")]
impl ToPredicate for InstitutionPredicate {
    fn to_predicate(&self) -> (&'static str, &(dyn ToSql + Sync)) {
        match self {
            Self::Id(u) | Self::MicrosoftEntraTenantId(u) => u.to_predicate(),
            Self::Name(s) => s.to_predicate(),
        }
    }
}

pub type InstitutionFilter = Filter<InstitutionPredicate>;

pub type InstitutionSortField = InstitutionField<OrderDirection, OrderDirection>;

impl OrderingField for InstitutionSortField {
    fn direction(self) -> OrderDirection {
        match self {
            Self::Id(d) | Self::Name(d) | Self::MicrosoftEntraTenantId(d) => d,
        }
    }
}

impl Default for InstitutionSortField {
    fn default() -> Self {
        Self::Name(OrderDirection::Desc)
    }
}

pub type InstitutionQuery = ComplexQuery<InstitutionFilter, InstitutionSortField>;
