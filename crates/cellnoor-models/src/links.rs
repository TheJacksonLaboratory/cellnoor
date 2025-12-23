use std::collections::HashMap;

use macro_attributes::json;

#[json]
#[serde(untagged)]
enum Link {
    One(String),
    Many(Vec<String>),
}

#[json]
pub struct Links(HashMap<String, Link>);

#[cfg(feature = "app")]
mod diesel_impls {
    use macros::impl_json_from_sql;

    use super::Links;
    use crate::utils::JsonFromSql;

    impl JsonFromSql for super::Links {}
    impl_json_from_sql!(Links);
}
