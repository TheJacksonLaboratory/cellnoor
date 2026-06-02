use std::fmt::Write;

use cellnoor_types::{
    filter::Filter,
    order_by::{OrderBy, OrderBySet},
    query::ComplexQuery,
};
use postgres_types::ToSql;

#[derive(Debug, Clone)]
pub struct Sql<'a>(pub(super) String, pub(super) Vec<&'a (dyn ToSql + Sync)>);

impl Sql<'_> {
    pub fn stmt(&self) -> &str {
        &self.0
    }

    pub fn params(&self) -> &[&(dyn ToSql + Sync)] {
        &self.1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SqlBuilder(&'static str);

impl SqlBuilder {
    pub const fn new(sql: &'static str) -> SqlBuilder {
        SqlBuilder(sql)
    }

    pub fn finish_with_params<'a>(&self, params: Vec<&'a (dyn ToSql + Sync)>) -> Sql<'a> {
        Sql(self.0.to_owned(), params)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterableSqlBuilder {
    prefix: &'static str,
    suffix: &'static str,
}

impl FilterableSqlBuilder {
    // We can use this function at compile-time to make sure every SQL statement has
    // "/* {where} */" in it!
    pub const fn new(base_sql: &'static str) -> Self {
        static WHERE_CLAUSE_PLACEHOLDER: &'static [u8] = b"/* {where} */";

        let mut i = 0;
        let mut sentinel_was_found = false;

        // The following nested while-loop is necessary because other string-searching
        // functionality is not const-compatible. This loop basically evaluates every
        // substring of sql with the same length as `WHERE_CLAUSE_SENTINEL`, checking if
        // said substring == WHERE_CLAUSE_SENTINEL
        while i <= base_sql.len() {
            let mut current_needle_idx = 0;

            while current_needle_idx < WHERE_CLAUSE_PLACEHOLDER.len() {
                let current_haystack_idx = i + current_needle_idx;
                let current_haystack_idx_is_valid = current_haystack_idx < base_sql.len();

                if !current_haystack_idx_is_valid {
                    break;
                }

                let haystack_char = base_sql.as_bytes()[current_haystack_idx];
                let needle_char = WHERE_CLAUSE_PLACEHOLDER[current_needle_idx];

                if haystack_char != needle_char {
                    break;
                }

                current_needle_idx += 1;
            }

            sentinel_was_found = current_needle_idx == WHERE_CLAUSE_PLACEHOLDER.len();
            if sentinel_was_found {
                break;
            }

            i += 1;
        }

        if !sentinel_was_found {
            panic!(r#"where-clause placeholder "/* {{where}} */"" not found in SQL statement"#);
        }

        let sentinel_idx = i;
        let (prefix, _) = base_sql.split_at(sentinel_idx);

        let suffix_idx = sentinel_idx + WHERE_CLAUSE_PLACEHOLDER.len();
        let (_, suffix) = base_sql.split_at(suffix_idx);

        Self { prefix, suffix }
    }

    pub fn finish_with_query<'a, P, O>(
        &self,
        ComplexQuery {
            filter,
            limit,
            offset,
            order_by,
        }: &'a ComplexQuery<P, O>,
    ) -> Sql<'a>
    where
        P: AsPredicate,
        O: Default + Copy + Into<&'static str>,
    {
        // Still tiny but should be more than enough inshallah
        let mut stmt = String::with_capacity(2048);
        let mut bind_params = Vec::with_capacity(32);

        // The base of the query
        stmt.push_str(self.prefix);

        // Write the where clause
        if let Some(filter) = filter {
            stmt.push_str(" where ");
            write_where_clause_predicates(&mut stmt, &mut bind_params, filter);
        }

        // Add the suffix
        stmt.push_str(self.suffix);

        // Write the order by clause
        stmt.push_str(" order by ");
        write_order_by_fields(&mut stmt, order_by);

        // Write limit clause
        if let Some(limit) = limit {
            bind_params.push(limit);
            write!(stmt, " limit ${}", bind_params.len()).unwrap();
        }

        // Write offset clause
        bind_params.push(offset);
        write!(stmt, " offset ${} ", bind_params.len()).unwrap();

        Sql(stmt, bind_params)
    }
}

pub trait AsPredicate {
    fn as_predicate(&self) -> (&'static str, (&'static str, &(dyn ToSql + Sync)));
}

fn write_where_clause_predicates<'a, 'b, P>(
    clause: &'a mut String,
    bind_params: &mut Vec<&'b (dyn ToSql + Sync)>,
    filter: &'b Filter<P>,
) -> &'a mut String
where
    P: AsPredicate,
{
    match filter {
        Filter::Leaf(pred) => {
            let (field, (operator, bind_param)) = pred.as_predicate();

            bind_params.push(bind_param);

            write!(clause, "{field} {operator} (${})", bind_params.len()).unwrap();
        }

        Filter::AllOf(filters) | Filter::AnyOf(filters) => {
            let (combinator, default) = if matches!(filter, Filter::AllOf(_)) {
                (" and ", "true")
            } else {
                (" or ", "false")
            };

            if filters.is_empty() {
                clause.push_str(default);
            }

            for (i, f) in filters.iter().enumerate() {
                if i != 0 {
                    clause.push_str(combinator);
                }

                clause.push('(');
                write_where_clause_predicates(clause, bind_params, f);
                clause.push(')');
            }
        }

        Filter::Not(filter) => {
            clause.push_str("not (");
            write_where_clause_predicates(clause, bind_params, filter);
            clause.push(')');
        }
    }

    clause
}

fn write_order_by_fields<'a, O>(
    clause: &'a mut String,
    order_by_set: &OrderBySet<O>,
) -> Option<&'a mut String>
where
    O: Default + Into<&'static str> + Copy,
{
    fn direction(desc: bool) -> &'static str {
        if desc { "desc" } else { "asc" }
    }

    match order_by_set {
        OrderBySet::One(OrderBy { field, desc }) => {
            let field: &str = field.clone().into();
            write!(clause, "{} {}", field, direction(*desc)).unwrap();
        }
        OrderBySet::Many(fields) => {
            for (i, order_by) in fields.iter().copied().enumerate() {
                if i != 0 {
                    clause.push_str(", ");
                }

                write_order_by_fields(clause, &OrderBySet::One(order_by));
            }
        }
    };

    Some(clause)
}

#[cfg(test)]
mod tests {
    use cellnoor_types::{
        filter::Filter,
        institution::{InstitutionPredicate, InstitutionQuery},
        operator::{SimpleStringOperator, UuidOperator},
    };
    use pretty_assertions::{assert_eq, assert_str_eq};
    use uuid::Uuid;

    use crate::db::stmt::write_where_clause_predicates;

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
        let pred1 = "((institution).name = ($1))";
        let pred2 = "((institution).id = ($2))";
        let all_of = format!("({pred1} and {pred2})");

        let pred3 = "((institution).id > ($3))";
        let not_pred = format!("(not {pred3})");

        let in_pred = "((institution).id = any ($4))";

        let expected_predicates = format!("{all_of} or {not_pred} or {in_pred}");

        let filter = complex_filter();
        let mut actual_where_clause = String::new();
        let mut bind_params = Vec::new();

        write_where_clause_predicates(&mut actual_where_clause, &mut bind_params, &filter);

        assert_str_eq!(expected_predicates, actual_where_clause);
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
            "order_by": {"field": "name", "desc": true}
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
