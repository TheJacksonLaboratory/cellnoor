use diesel::{
    define_sql_function,
    sql_types::{Array, Text},
};

define_sql_function! { fn like_any(string: Text, patterns: Array<Text>) -> Bool }

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

#[macro_export]
macro_rules! with_eq_any {
    ($filter:ident.and($($col:path = $values:expr),*)) => {{
        $(
            if !$values.is_empty() {
                $filter = $filter.and_filter($col.assume_not_null().eq_any($values));
            }
        )*

        $filter
    }};
}

#[macro_export]
macro_rules! with_eq {
    ($filter:ident.and($($col:path = $values:expr),*)) => {{
        $(
            if let Some(value) = $value {
                $stmt = $stmt.filter($col.eq(value));
            }
        )*

        $stmt
    }};
}

#[macro_export]
macro_rules! with_like {
    ($filter:ident.and($($col:path = $string:expr),*)) => {{
        $(
            if let Some(s) = $string {
                $filter = $filter.and_filter($col.assume_not_null().like(s))
            }
        )*

        $filter
    }};
}

#[macro_export]
macro_rules! apply_jsonb_contains_filters {
    ($stmt:expr, filters = {$($col:expr => $value:expr),*}) => {{
        $(
            let mut jsonb_condition: Option<Box<dyn diesel::BoxableExpression<_, diesel::pg::Pg, SqlType = diesel::sql_types::Nullable<diesel::sql_types::Bool>>>> = None;

            for jsonb in $value {
                if let Some(condition) = jsonb_condition {
                    jsonb_condition = Some(Box::new(condition.or($col.contains(jsonb))));
                } else {
                    jsonb_condition = Some(Box::new($col.contains(jsonb)));
                }
            }

            if let Some(condition) = jsonb_condition {
                $stmt = $stmt.filter(condition);
            }
        )*

        $stmt
    }};
}
