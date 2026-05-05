use macro_attributes::base_model;
#[cfg(feature = "postgres-types")]
use postgres_types::ToSql;

#[cfg(feature = "postgres-types")]
use crate::query::order_by::OrderingField;
use crate::query::{filter::ToPredicate, order_by::OrderBy};

pub mod filter;
pub mod order_by;

#[base_model]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct Query<F, O>
where
    O: Default,
{
    filter: Option<F>,
    limit: Option<i32>,
    offset: i32,
    order_by: OrderBy<O>,
    pub detailed: bool,
}

impl<F, O> Default for Query<F, O>
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

#[cfg(feature = "postgres-types")]
impl<P, O> Query<filter::Filter<P>, O>
where
    P: AsRef<str> + ToPredicate,
    O: Default + Copy + AsRef<str> + OrderingField,
{
    pub fn to_sql_query(&self) -> (String, Vec<&(dyn ToSql + Sync)>) {
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

        sql.push_str(&order_by.to_order_by_clause());

        if let Some(limit) = limit {
            sql.push_str(&format!(" limit {limit}"));
        }

        // Note the final space
        sql.push_str(&format!(" offset {offset} "));

        (sql, params)
    }
}
