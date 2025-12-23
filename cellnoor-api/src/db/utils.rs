use diesel::{
    define_sql_function,
    sql_types::{Array, Text},
};

define_sql_function! { fn like_any(string: Text, patterns: Array<Text>) -> Bool }
