use macro_attributes::base_model;
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

use crate::query::order_by::OrderBy;
#[cfg(feature = "postgres-types")]
use crate::query::{filter::ToPredicate, order_by::OrderingField};

pub mod filter;
pub mod order_by;

#[base_model]
#[derive(Copy, Default)]
#[cfg_attr(feature = "serde", serde(default))]
pub struct SimpleQuery<O>
where
    O: Default,
{
    pub limit: Option<i32>,
    pub offset: i32,
    pub detailed: bool,
    pub order_by: O,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct ComplexQuery<F, O>
where
    O: Default,
{
    pub filter: Option<F>,
    pub limit: Option<i32>,
    pub offset: i32,
    pub order_by: OrderBy<O>,
    pub detailed: bool,
}

impl<F, O> Default for ComplexQuery<F, O>
where
    O: Default,
{
    fn default() -> Self {
        Self {
            filter: None,
            limit: None,
            offset: 0,
            order_by: OrderBy::default(),
            detailed: false,
        }
    }
}

impl<P, O> ComplexQuery<filter::Filter<P>, O>
where
    P: Into<filter::Filter<P>>,
    O: Default,
{
    pub fn from_filter(predicate: P, detailed: bool) -> Self {
        Self {
            filter: Some(predicate.into()),
            detailed,
            ..Default::default()
        }
    }

    pub fn from_simple_query(
        SimpleQuery {
            limit,
            offset,
            detailed,
            order_by,
        }: SimpleQuery<O>,
    ) -> Self {
        Self {
            limit,
            offset,
            detailed,
            order_by: OrderBy::OneField(order_by),
            ..Default::default()
        }
    }
}

#[cfg(feature = "postgres-types")]
impl<P, O> ComplexQuery<filter::Filter<P>, O>
where
    P: AsRef<str> + ToPredicate,
    O: Default + Copy + AsRef<str> + OrderingField,
{
    pub fn to_sql_query_with_group_by(&self, group_by: &str) -> (String, Vec<&(dyn ToSql + Sync)>) {
        let Self {
            filter,
            limit,
            offset,
            order_by,
            detailed: _,
        } = self;

        // That's probably enough for a where clause
        let mut sql = String::with_capacity(1024);

        let (where_clause, params) = if let Some(filter) = filter {
            filter.to_where_clause()
        } else {
            (String::new(), Vec::new())
        };

        sql.push_str(&where_clause);

        sql.push(' ');

        if !group_by.is_empty() {
            sql.push_str(group_by);
            sql.push(' ');
        }

        sql.push_str(&order_by.to_order_by_clause());

        if let Some(limit) = limit {
            sql.push_str(&format!(" limit {limit}"));
        }

        // Note the final space
        sql.push_str(&format!(" offset {offset} "));

        (sql, params)
    }

    pub fn to_sql_query(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
        self.to_sql_query_with_group_by("")
    }
}
