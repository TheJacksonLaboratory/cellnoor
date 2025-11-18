use std::collections::HashMap;

use macro_attributes::base_model;

#[base_model]
#[cfg_attr(
    feature = "app",
    derive(diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)
)]
#[cfg_attr(feature = "app", diesel(sql_type = diesel::sql_types::Jsonb))]
pub struct Links(HashMap<String, String>);

#[cfg(feature = "app")]
mod diesel_impls {
    use macros::impl_json_from_sql;

    use super::Links;
    use crate::utils::JsonFromSql;

    impl JsonFromSql for super::Links {}
    impl_json_from_sql!(Links);
}
