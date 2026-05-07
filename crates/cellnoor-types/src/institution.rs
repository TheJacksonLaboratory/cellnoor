use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
pub use query::{InstitutionFilter, InstitutionOrderBy, InstitutionPredicate, InstitutionQuery};
use uuid::Uuid;

use crate::simple_links::SimpleLinks;

mod query;

#[base_model]
pub struct NewInstitution {
    pub name: NonemptyString,
    pub microsoft_entra_tenant_id: Uuid,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "institution"))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct InstitutionRecord {
    pub id: Uuid,
    pub name: NonemptyString,
    pub microsoft_entra_tenant_id: Uuid,
}

#[base_model]
pub struct Institution {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: InstitutionRecord,
    pub links: SimpleLinks,
}

impl Institution {
    pub fn from_record(record: InstitutionRecord) -> Self {
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
        institution::{InstitutionFilter, InstitutionQuery},
        query::filter::UuidOperator,
    };

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
        let expected_query_string =
            "filter[any_of][0][all_of][0][name]=Jackson+Laboratory&\
             filter[any_of][0][all_of][1][id][eq]=00000000-0000-0000-0000-000000000000&\
             filter[any_of][1][not][id][gt]=ffffffff-ffff-ffff-ffff-ffffffffffff&\
             filter[any_of][2][id][in][0]=00000000-0000-0000-0000-000000000000&\
             filter[any_of][2][id][in][1]=ffffffff-ffff-ffff-ffff-ffffffffffff&limit=10&offset=0&\
             order_by[name]=desc&detailed=false";

        let filter = complex_filter();
        let query = InstitutionQuery {
            filter: Some(filter),
            limit: Some(10),
            offset: 0,
            ..Default::default()
        };

        let config = serde_qs::Config::new().max_depth(10);

        let actual_query_string = config.serialize_string(&query).unwrap();

        assert_str_eq!(expected_query_string, actual_query_string);

        let deserialized_query = config.deserialize_str(&expected_query_string).unwrap();

        assert_eq!(query, deserialized_query);
    }
}
