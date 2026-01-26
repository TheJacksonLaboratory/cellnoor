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
