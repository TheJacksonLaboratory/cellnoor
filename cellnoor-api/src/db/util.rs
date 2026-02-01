use std::ops::{Deref, DerefMut};

use diesel::{
    define_sql_function,
    sql_types::{Array, Text},
};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::deadpool::{Object, Pool},
};

define_sql_function! { fn like_any(string: Text, patterns: Array<Text>) -> Bool }

pub type DbConnectionPool = Pool<AsyncPgConnection>;

#[derive(aide::OperationIo)]
pub struct DbConnection(Object<AsyncPgConnection>);

impl DbConnection {
    pub fn new(connection: Object<AsyncPgConnection>) -> Self {
        Self(connection)
    }
}

impl Deref for DbConnection {
    type Target = Object<AsyncPgConnection>;

    fn deref(&self) -> &Object<AsyncPgConnection> {
        &self.0
    }
}

impl DerefMut for DbConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn jiff_diesel_tuple_to_jiff(
    (t1, t2): (jiff_diesel::Timestamp, jiff_diesel::Timestamp),
) -> (jiff::Timestamp, jiff::Timestamp) {
    (t1.to_jiff(), t2.to_jiff())
}

pub fn jiff_diesel_optional_tuple_to_jiff(
    (t1, t2): (jiff_diesel::Timestamp, Option<jiff_diesel::Timestamp>),
) -> (jiff::Timestamp, Option<jiff::Timestamp>) {
    (t1.to_jiff(), t2.map(jiff_diesel::Timestamp::to_jiff))
}
