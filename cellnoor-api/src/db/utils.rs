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

pub type DbConnection = Object<AsyncPgConnection>;
