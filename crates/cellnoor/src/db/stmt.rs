use std::fmt::Write;

use cellnoor_types::{
    filter::{AsPredicate, Filter, Predicate},
    order_by::{OrderBy, OrderBySet},
    query::{ComplexQuery, OrderField},
};
use postgres_types::ToSql;

#[derive(Debug, Clone)]
pub struct Sql<'a>(pub(super) String, pub(super) Vec<&'a (dyn ToSql + Sync)>);

impl<'a> Sql<'a> {
    pub fn new(stmt: &str, params: Vec<&'a (dyn ToSql + Sync)>) -> Self {
        Self(stmt.to_owned(), params)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterableSqlBuilder {
    prefix: &'static str,
    suffix: &'static str,
}

// `str::find` is not const, so scan for the placeholder by hand to keep the
// check at compile time.
const fn placeholder_index(base_sql: &str, placeholder: &str) -> usize {
    let (haystack, needle) = (base_sql.as_bytes(), placeholder.as_bytes());

    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        let mut i = 0;
        while i < needle.len() && haystack[start + i] == needle[i] {
            i += 1;
        }

        if i == needle.len() {
            return start;
        }

        start += 1;
    }

    panic!(r#"where-clause placeholder "/* {{where}} */" not found in SQL statement"#);
}

impl FilterableSqlBuilder {
    // We can use this function at compile-time to make sure every SQL statement
    // has "/* {where} */" in it!
    pub const fn new(base_sql: &'static str) -> Self {
        const PLACEHOLDER: &str = "/* {where} */";

        let index = placeholder_index(base_sql, PLACEHOLDER);
        let (prefix, rest) = base_sql.split_at(index);
        let (_, suffix) = rest.split_at(PLACEHOLDER.len());

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
        O: OrderField,
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

fn write_where_clause_predicates<'a, P>(
    clause: &mut String,
    bind_params: &mut Vec<&'a (dyn ToSql + Sync)>,
    filter: &'a Filter<P>,
) where
    P: AsPredicate,
{
    match filter {
        Filter::Leaf(predicate) => {
            let Predicate {
                relation,
                column,
                operator,
                value,
            } = predicate.as_predicate();

            bind_params.push(value);

            write_column(clause, relation, column);
            write!(clause, " {operator} (${})", bind_params.len()).unwrap();
        }

        // An empty set of predicates is the identity of its combinator
        Filter::AllOf(filters) => write_combined(clause, bind_params, filters, " and ", "true"),
        Filter::AnyOf(filters) => write_combined(clause, bind_params, filters, " or ", "false"),

        Filter::Not(filter) => {
            clause.push_str("not (");
            write_where_clause_predicates(clause, bind_params, filter);
            clause.push(')');
        }
    }
}

fn write_combined<'a, P>(
    clause: &mut String,
    bind_params: &mut Vec<&'a (dyn ToSql + Sync)>,
    filters: &'a [Filter<P>],
    combinator: &str,
    identity: &str,
) where
    P: AsPredicate,
{
    if filters.is_empty() {
        clause.push_str(identity);
        return;
    }

    for (i, filter) in filters.iter().enumerate() {
        if i != 0 {
            clause.push_str(combinator);
        }

        clause.push('(');
        write_where_clause_predicates(clause, bind_params, filter);
        clause.push(')');
    }
}

fn write_order_by_fields<O>(clause: &mut String, order_by_set: &OrderBySet<O>)
where
    O: OrderField,
{
    for (i, OrderBy { field, desc }) in order_by_set.iter().enumerate() {
        if i != 0 {
            clause.push_str(", ");
        }

        write_column(clause, O::RELATION, field.as_ref());
        clause.push_str(if desc { " desc" } else { " asc" });
    }
}

fn write_column(clause: &mut String, relation: &str, column: &str) {
    write!(clause, "({relation}).{column}").unwrap();
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
