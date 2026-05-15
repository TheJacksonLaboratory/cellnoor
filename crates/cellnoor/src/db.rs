pub use client::{Client, Pool, Transaction, User};
pub use delete::delete_by_id;
pub use insert::{insert_into, insert_into_no_returning};
use postgres_types::ToSql;
pub use select::{construct_select_stmt, select_one};
pub use update::update;

mod client;
mod delete;
mod insert;
mod select;
mod update;

type FieldValuePair<'a, F> = (F, &'a (dyn ToSql + Sync));

pub type Record<'a, F, const N: usize> = [FieldValuePair<'a, F>; N];

pub trait ToRecord<F, const N: usize> {
    fn to_record(&self) -> Record<'_, F, N>;
}
