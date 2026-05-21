pub use client::{Client, Pool, Transaction, User};
pub use delete::delete_by_id;
pub use insert::{insert_into, insert_into_no_returning};
use postgres_types::ToSql;
pub use select::select_one;
pub use stmt::{AsPredicate, BaseSqlStmt, Sql};
pub use update::update;

mod client;
mod delete;
mod insert;
mod select;
mod stmt;
mod update;

type FieldValuePair<'a, F> = (F, &'a (dyn ToSql + Sync));

pub type FieldValuePairs<'a, F, const N: usize> = [FieldValuePair<'a, F>; N];

pub trait AsFieldValuePairs<F, const N: usize> {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, F, N>;
}
