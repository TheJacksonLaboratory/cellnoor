use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::InstitutionQuery;
use uuid::Uuid;

#[base_model]
pub struct NewInstitution {
    pub name: NonemptyString,
    pub microsoft_entra_tenant_id: Uuid,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "institution"))]
pub struct Institution {
    pub id: Uuid,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub name: NonemptyString,
    pub microsoft_entra_tenant_id: Uuid,
}

mod query {
    use macro_attributes::field_enum;
    #[cfg(feature = "postgres-types")]
    use postgres_types::ToSql;

    #[cfg(feature = "postgres-types")]
    use crate::query::filter::AsPredicate;
    use crate::query::{
        Query,
        filter::{Filter, StringOperator, UuidOperator},
        order_by::{OrderDirection, OrderingField},
    };
    #[field_enum]
    #[strum(prefix = "institution.")]
    pub enum InstitutionField<U, S> {
        Id(U),
        Name(S),
        MicrosoftEntraTenantId(U),
    }

    pub type InstitutionPredicate = InstitutionField<UuidOperator, StringOperator>;

    #[cfg(feature = "postgres-types")]
    impl AsPredicate for InstitutionPredicate {
        fn as_predicate(&self) -> (&'static str, &dyn ToSql) {
            match self {
                Self::Id(u) | Self::MicrosoftEntraTenantId(u) => u.as_predicate(),
                Self::Name(s) => s.as_predicate(),
            }
        }
    }

    pub type InstitutionFilter = Filter<InstitutionPredicate>;

    pub type InstitutionOrderBy = InstitutionField<OrderDirection, OrderDirection>;

    impl OrderingField for InstitutionOrderBy {
        fn direction(self) -> OrderDirection {
            match self {
                Self::Id(d) | Self::Name(d) | Self::MicrosoftEntraTenantId(d) => d,
            }
        }
    }

    impl Default for InstitutionOrderBy {
        fn default() -> Self {
            Self::Name(OrderDirection::Desc)
        }
    }

    pub type InstitutionQuery = Query<InstitutionFilter, InstitutionOrderBy>;
}

#[cfg(all(feature = "postgres-types", test))]
mod tests {
    use pretty_assertions::assert_str_eq;
    use uuid::Uuid;

    use super::query::InstitutionPredicate;
    use crate::query::filter::{Filter, ScalarOperator, UuidOperator};

    #[test]
    fn test_where_clause_construction() {
        let pred1 =
            InstitutionPredicate::Name(ScalarOperator::Eq("Jackson Laboratory".to_owned()).into());
        let pred2 = InstitutionPredicate::Id(UuidOperator::Eq(Uuid::nil()));
        let all_of = Filter::AllOf(vec![pred1.into(), pred2.into()]);

        let pred3 = InstitutionPredicate::Id(UuidOperator::Gt(Uuid::max())).into();
        let not = Filter::Not(Box::new(pred3));

        let in_pred = InstitutionPredicate::Id(UuidOperator::In(vec![Uuid::nil(), Uuid::max()]));

        let filter = Filter::AnyOf(vec![all_of, not, in_pred.into()]);

        let expected_query = "((institution.name = ($1)) and (institution.id = ($2))) or (not \
                              (institution.id > ($3))) or (institution.id = any ($4))";

        let (actual_query, _actual_bind_params) = filter.as_where_clause();

        assert_str_eq!(expected_query, actual_query);
    }
}
