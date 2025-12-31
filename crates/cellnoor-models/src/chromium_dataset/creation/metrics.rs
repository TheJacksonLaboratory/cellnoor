use std::collections::HashMap;

use macro_attributes::json;
use macros::{impl_json_from_sql, impl_json_to_sql};
use serde_json::Value;

#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

pub mod multi_row_csv;

#[json]
#[serde(untagged)]
pub enum ParsedMetricsData {
    KeyValue(HashMap<String, Value>),
    Tabular(Vec<multi_row_csv::Row>),
}

#[cfg(feature = "app")]
impl JsonFromSql for ParsedMetricsData {}
impl_json_from_sql!(ParsedMetricsData);

#[cfg(feature = "app")]
impl JsonToSql for ParsedMetricsData {}
impl_json_to_sql!(ParsedMetricsData);
