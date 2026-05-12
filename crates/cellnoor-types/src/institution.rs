use macro_attributes::base_model;
pub use query::{InstitutionPredicate, InstitutionQuery, SimpleInstitutionQuery};

use crate::{
    id::{Id, NoId},
    institution::record::InstitutionRecord,
    simple_links::SimpleLinks,
};

mod query;

mod record {
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use uuid::Uuid;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "institution"))]
    #[cfg_attr(feature = "schemars", schemars(inline))]
    pub struct InstitutionRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub name: NonemptyString,
        pub microsoft_entra_tenant_id: Uuid,
    }
}

pub type NewInstitution = InstitutionRecord<NoId>;

pub type SavedInstitution = InstitutionRecord<Id>;

#[base_model]
#[cfg_attr(feature = "schemars", schemars(rename = "Institution"))]
pub struct Institution {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedInstitution,
    pub links: SimpleLinks,
}

impl Institution {
    pub fn from_record(record: SavedInstitution) -> Self {
        Self {
            links: SimpleLinks {
                self_: format!("/institutions/{}", record.id),
            },
            record,
        }
    }
}

#[cfg(all(feature = "postgres-types", feature = "serde", test))]
mod tests {
    use pretty_assertions::{assert_eq, assert_str_eq};
    use uuid::Uuid;

    use super::query::InstitutionPredicate;
    use crate::{
        SimpleStringOperator,
        institution::InstitutionQuery,
        query::filter::{Filter, UuidOperator},
    };

    type InstitutionFilter = Filter<InstitutionPredicate>;

    fn complex_filter() -> InstitutionFilter {
        let pred1 = InstitutionPredicate::Name(
            SimpleStringOperator::ImplicitEq("Jackson Laboratory".to_owned()).into(),
        );
        let pred2 = InstitutionPredicate::Id(UuidOperator::Eq(Uuid::nil()));
        let all_of = InstitutionFilter::AllOf(vec![pred1.into(), pred2.into()]);

        let pred3 = InstitutionPredicate::Id(UuidOperator::Gt(Uuid::max())).into();
        let not_pred = InstitutionFilter::Not(Box::new(pred3));

        let in_pred = InstitutionPredicate::Id(UuidOperator::In(vec![Uuid::nil(), Uuid::max()]));

        InstitutionFilter::AnyOf(vec![all_of, not_pred, in_pred.into()])
    }

    #[test]
    fn where_clause_construction() {
        let expected_where_clause = "where (((institution).name = ($1)) and ((institution).id = \
                                     ($2))) or (not ((institution).id > ($3))) or \
                                     ((institution).id = any ($4))";

        let (actual_where_clause, _actual_bind_params) = complex_filter().to_where_clause();

        assert_str_eq!(expected_where_clause, actual_where_clause);
    }

    #[test]
    fn complex_query_serialization() {
        let expected_query = serde_json::json!({
            "filter": {
                "any_of": [
                    {
                        "all_of": [
                            {"name": "Jackson Laboratory"},
                            {"id": {"eq": Uuid::nil()}}
                        ]
                    },
                    {
                        "not": {
                            "id": {
                                "gt": Uuid::max()
                            }
                        }
                    },
                    {
                        "id": {
                            "in": [
                                Uuid::nil(),
                                Uuid::max()
                            ]
                        }
                    }
                ]
            },
            "limit": 10,
            "offset": 0,
            "order_by": {"field": "name", "desc": true},
            "detailed": false
        });

        let filter = complex_filter();
        let query = InstitutionQuery {
            filter: Some(filter),
            limit: Some(10),
            offset: 0,
            ..Default::default()
        };

        let actual_query = serde_json::to_value(query).unwrap();

        assert_eq!(expected_query, actual_query);
    }
}
