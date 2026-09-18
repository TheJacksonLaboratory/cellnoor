pub use client::{Client, Pool, Transaction};
pub use error::DbError;
use postgres_types::ToSql;
pub use stmt::{FilterableSqlBuilder, Sql};

mod client;
mod error;
mod insert;
mod stmt;
#[cfg(test)]
pub mod test_utils;
mod update;

/// The fieldnames of a row paired with their values.
pub type FieldValues<'a, F> = Vec<(F, &'a (dyn ToSql + Sync))>;

/// [`FieldValues`] with every fieldname converted to a `&'static str`
type Columns<'a> = Vec<(&'static str, &'a (dyn ToSql + Sync))>;

/// [`Columns`] as the statement builders read it.
type ColumnSlice<'a> = [(&'static str, &'a (dyn ToSql + Sync))];

/// A row that can be written to the relation it belongs to.
///
/// `Field` is the relation's field enum where a query filters on it, and
/// `&'static str` for the join tables that nothing filters on.
pub trait Insert: cellnoor_types::Relation {
    type Field: Into<&'static str>;

    fn fields(&self) -> FieldValues<'_, Self::Field>;
}

fn columns<T>(record: &T) -> Columns<'_>
where
    T: Insert,
{
    record
        .fields()
        .into_iter()
        .map(|(field, value)| (field.into(), value))
        .collect()
}
