use std::collections::HashMap;

use macro_attributes::{insert_select, json};
use macros::{impl_json_from_sql, impl_json_to_sql};
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::chromium_datasets;
use serde_json::{Number, Value};
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{JsonFromSql, JsonToSql};

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetFields {
    name: NonEmptyString,
    lab_id: Uuid,
    data_path: NonEmptyString,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct MetricsFile {
    pub(super) filename: NonEmptyString,
    pub(super) raw_contents: String,
}

#[json]
#[serde(tag = "format")]
pub enum ParsedMetrics {
    Json {
        #[serde(flatten)]
        file: MetricsFile,
        parsed_data: HashMap<String, Value>,
    },
    SingleRowCsv {
        #[serde(flatten)]
        file: MetricsFile,
        parsed_data: HashMap<String, Number>,
    },
    MultiRowCsv {
        files: Vec<ParsedMultiRowCsv>,
    },
}

#[cfg(feature = "app")]
impl JsonFromSql for ParsedMetrics {}
impl_json_from_sql!(ParsedMetrics);

#[cfg(feature = "app")]
impl JsonToSql for ParsedMetrics {}
impl_json_to_sql!(ParsedMetrics);

#[json]
pub struct ParsedMultiRowCsv {
    #[serde(flatten)]
    pub(super) file: MetricsFile,
    pub(super) parsed_data: Vec<super::creation::metrics::multi_row_csv::Row>,
}
