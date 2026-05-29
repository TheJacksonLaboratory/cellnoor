use std::collections::HashMap;

use csvranger::TenxCsvValue;
use macro_attributes::json;
use macros::{impl_json_from_sql, impl_json_to_sql};

#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

pub mod multi_row_csv;

#[json]
#[serde(untagged)]
pub enum ParsedMetricsData {
    KeyValue(HashMap<String, TenxCsvValue>),
    Tabular(Vec<multi_row_csv::Row>),
    Tabular2(Vec<HashMap<String, TenxCsvValue>>),
}

#[cfg(feature = "app")]
impl JsonFromSql for ParsedMetricsData {}
impl_json_from_sql!(ParsedMetricsData);

#[cfg(feature = "app")]
impl JsonToSql for ParsedMetricsData {}
impl_json_to_sql!(ParsedMetricsData);
