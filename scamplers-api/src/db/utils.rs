use diesel::{
    define_sql_function,
    sql_types::{Array, Text},
};

define_sql_function! { fn like_any(string: Text, patterns: Array<Text>) -> Bool }

// You will, from time-to-time, have the urge to get rid of this macro, opting
// to actually type everything out. Don't. do that. It's not that much code, but
// it's needlessly repetetive, so you actually end up saving a lot of code this
// way.
#[macro_export]
macro_rules! query {
    ($select:ident::query($query:ident).order_by($($enum_variant:path = $db_col:expr),*)) => {
        {
            use $crate::db::ToBoxedFilter;

            let mut stmt = $select::query()
            .limit($query.limit())
            .offset($query.offset())
            .into_boxed();

            if let Some(filter) = $query.filter() {
                stmt = stmt.filter(filter.to_boxed_filter());
            }

            for ordering in $query.order_by() {
                stmt = match (ordering.field(), ordering.descending()) {
                    $(
                        ($enum_variant, true) => stmt.then_order_by($db_col.desc()),
                        ($enum_variant, _) => stmt.then_order_by($db_col.asc()),
                    )*
                }
            }
            stmt
        }
    };
}
