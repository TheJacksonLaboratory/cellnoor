use std::fmt::Write;

use cellnoor_types::{ToPredicate, query::ComplexQuery};
pub use client::{Client, Pool, Transaction, User};
pub use delete::delete_by_id;
pub use insert::{insert_into, insert_into_no_returning};
use postgres_types::ToSql;
pub use select::select_one;
pub use update::update;
use uuid::Uuid;

use crate::error::ErrorInner;

mod client;
mod delete;
mod insert;
mod select;
mod update;

type FieldValuePair<'a, F> = (F, &'a (dyn ToSql + Sync));

pub type FieldValuePairs<'a, F, const N: usize> = [FieldValuePair<'a, F>; N];

pub trait AsFieldValuePairs<F, const N: usize> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, F, N>;
}

pub struct SqlTemplate(&'static str);

impl SqlTemplate {
    pub fn new(template: &'static str) -> Self {
        Self(template)
    }

    pub fn finish_with_params<'a>(self, params: Vec<&'a (dyn ToSql + Sync)>) -> Sql<'a> {
        Sql(self.0.to_owned(), params)
    }

    pub fn finish_with_query<P, O>(self, query: &ComplexQuery<P, O>) -> Result<Sql<'_>, ErrorInner>
    where
        P: ToPredicate + AsRef<str>,
        O: Default + Copy + AsRef<str>,
    {
        let base = self.0;
        if !base.contains("where true") {
            return Err(ErrorInner::Other {
                message: format!("'where true' not found in base statement: {base}"),
                sql_state: None,
            });
        }

        let (where_clause, params) = query.to_where_clause();
        let mut stmt = base.replace("where true", &where_clause);

        write!(
            stmt,
            "{} {} {}",
            query.to_order_by_clause(),
            query.limit_clause(),
            query.offset_clause()
        )
        .map_err(|e| ErrorInner::Other {
            message: e.to_string(),
            sql_state: None,
        })?;

        Ok(Sql(stmt, params))
    }
}

pub struct Sql<'a>(String, Vec<&'a (dyn ToSql + Sync)>);
