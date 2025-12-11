use std::collections::HashMap;

use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::chromium_datasets;
use serde_json::{Number, Value};
use uuid::Uuid;

use super::creation::metrics::multi_row_csv;
#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetFields {
    pub(super) name: NonEmptyString,
    pub(super) lab_id: Uuid,
    pub(super) data_path: NonEmptyString,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct RawMetricsFile {
    pub(super) filename: NonEmptyString,
    pub(super) raw_contents: String,
}

#[cfg(not(feature = "typescript"))]
#[json]
pub struct ParsedMetricsFile<D> {
    #[serde(flatten)]
    pub(super) file: RawMetricsFile,
    pub(super) parsed_data: D,
}

#[cfg(feature = "typescript")]
#[json]
pub struct ParsedMetricsFile<D>
where
    D: ts_rs::TS,
    <D as ts_rs::TS>::OptionInnerType: ts_rs::TS,
{
    #[serde(flatten)]
    pub(super) file: RawMetricsFile,
    pub(super) parsed_data: D,
}

#[json]
#[serde(tag = "format")]
pub enum ParsedMetrics {
    Json(Vec<ParsedMetricsFile<HashMap<String, Value>>>),
    SingleRowCsv(Vec<ParsedMetricsFile<HashMap<String, Number>>>),
    MultiRowCsv(Vec<ParsedMetricsFile<Vec<multi_row_csv::Row>>>),
}

#[cfg(feature = "app")]
impl JsonFromSql for ParsedMetrics {}
impl_json_from_sql!(ParsedMetrics);

#[cfg(feature = "app")]
impl JsonToSql for ParsedMetrics {}
impl_json_to_sql!(ParsedMetrics);
