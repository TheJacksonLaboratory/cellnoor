pub use client::{Client, Pool, Transaction, User};
pub use insert::insert_into;
use postgres_types::ToSql;
pub use select::{construct_select_stmt, select_one};
pub use update::update;

mod client;
mod delete;
mod insert;
mod select;
mod table;
mod update;

type FieldValuePair<'a, F> = (F, &'a (dyn ToSql + Sync));

pub type Record<'a, F, const N: usize> = [FieldValuePair<'a, F>; N];

pub trait ToRecord<F, const N: usize> {
    fn to_record(&self) -> Record<F, N>;
}
