pub use client::{Client, Pool, Transaction};
use postgres_types::ToSql;
pub use stmt::{FilterableSqlBuilder, Sql, SqlBuilder};

mod client;
mod insert;
mod stmt;
#[cfg(test)]
pub mod test_utils;
mod update;

/// The columns of one row, paired with the values to bind to them.
pub type FieldValues<'a, F> = Vec<(F, &'a (dyn ToSql + Sync))>;

/// [`FieldValues`] as the statement builders read it.
type FieldValueSlice<'a, F> = [(F, &'a (dyn ToSql + Sync))];

/// A row that can be written to the relation it belongs to.
///
/// `Field` is the relation's field enum where a query filters on it, and
/// `&'static str` for the join tables that nothing filters on.
pub trait Insert: cellnoor_types::Relation {
    type Field: AsRef<str>;

    fn fields(&self) -> FieldValues<'_, Self::Field>;
}
