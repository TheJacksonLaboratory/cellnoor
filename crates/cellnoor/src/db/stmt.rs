use std::fmt::Write;

use cellnoor_types::{
    filter::Filter,
    order_by::{OrderBy, OrderBySet},
    query::ComplexQuery,
};
use postgres_types::ToSql;

use crate::error::ErrorInner;

pub struct BaseSqlStmt(&'static str);

impl BaseSqlStmt {
    pub fn new(sql: &'static str) -> Self {
        Self(sql)
    }

    pub fn finish_with_params<'a>(self, params: Vec<&'a (dyn ToSql + Sync)>) -> Sql<'a> {
        Sql(self.0.to_owned(), params)
    }

    pub fn finish_with_query<P, O>(
        self,
        ComplexQuery {
            filter,
            limit,
            offset,
            order_by,
        }: &ComplexQuery<P, O>,
    ) -> Result<Sql<'_>, ErrorInner>
    where
        P: AsPredicate,
        O: Default + Copy + AsRef<str>,
    {
        let base = self.0;
        let base_is_compatible_with_filter = base.contains("where true");

        let (mut stmt, bind_params) = match filter {
            Some(filter) if base_is_compatible_with_filter => {
                // Arbitrary number of bytes that's really small but more than enough for a
                // where clause
                let mut where_clause = String::with_capacity(1024);
                where_clause.push_str("where ");
                let mut bind_params = Vec::with_capacity(32);

                write_where_clause_predicates(&mut where_clause, &mut bind_params, filter);

                (base.replace("where true", &where_clause), bind_params)
            }
            Some(_) => {
                return Err(ErrorInner::Other {
                    message: format!(
                        "'where true' not found in base statement, so a 'where' clause cannot be \
                         substituted in: {base}"
                    ),
                    sql_state: None,
                });
            }
            None => (base.to_owned(), Vec::new()),
        };

        stmt.push_str(" order by ");

        write_order_by_fields(&mut stmt, order_by);

        if let Some(limit) = limit {
            write!(stmt, " limit {limit}").unwrap();
        }
        write!(stmt, " offset {offset}").unwrap();

        Ok(Sql(stmt, bind_params))
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

            clause.push_str(field);
            clause.push(' ');
            clause.push_str(operator);
            clause.push_str(" ($");
            clause.push_str(&bind_params.len().to_string());
            clause.push(')');
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
    O: Default + AsRef<str> + Copy,
{
    fn direction(desc: bool) -> &'static str {
        if desc { "desc" } else { "asc" }
    }

    match order_by_set {
        OrderBySet::One(OrderBy { field, desc }) => {
            clause.push_str(field.as_ref());
            clause.push(' ');
            clause.push_str(direction(*desc));
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

pub struct Sql<'a>(pub(super) String, pub(super) Vec<&'a (dyn ToSql + Sync)>);

impl Sql<'_> {
    pub fn stmt(&self) -> &str {
        &self.0
    }

    pub fn params(&self) -> &[&(dyn ToSql + Sync)] {
        &self.1
    }
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
