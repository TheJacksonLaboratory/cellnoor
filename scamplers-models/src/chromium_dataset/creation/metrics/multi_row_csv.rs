use serde::{Deserialize, de};
use serde_json::Value;

use crate::chromium_dataset::{
    common::{ParsedMetrics, ParsedMetricsFile, RawMetricsFile},
    creation::metrics::common::parse_str_as_number,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(optional_fields))]
pub struct MultiRowCsv {
    file: RawMetricsFile,
    #[cfg_attr(feature = "typescript", ts(skip))]
    parsed_data: Vec<Row>,
}

impl From<Vec<MultiRowCsv>> for ParsedMetrics {
    fn from(files: Vec<MultiRowCsv>) -> Self {
        ParsedMetrics::MultiRowCsv {
            files: files
                .into_iter()
                .map(|MultiRowCsv { file, parsed_data }| ParsedMetricsFile { file, parsed_data })
                .collect(),
        }
    }
}

impl<'de> Deserialize<'de> for MultiRowCsv {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let file = RawMetricsFile::deserialize(deserializer)?;
        let mut csv = csv::Reader::from_reader(file.raw_contents.as_bytes());
        let headers = csv.headers().map_err(de::Error::custom)?.clone();

        let mut parsed_data = Vec::with_capacity(100);
        for record in csv.records() {
            let record = record.map_err(de::Error::custom)?;
            let simple_fields: SimpleFields = record
                .deserialize(Some(&headers))
                .map_err(de::Error::custom)?;

            let metric_value_str = record
                .get(5)
                .ok_or(de::Error::missing_field("Metric Value"))?;
            let extracted_metric_value = match metric_value_str.split_once(' ') {
                Some((actual_value, _)) => actual_value,
                None => metric_value_str,
            };

            let metric_value = parse_str_as_number(extracted_metric_value).map_or_else(
                |_| Value::String(metric_value_str.to_owned()),
                Value::Number,
            );

            parsed_data.push(Row {
                simple_fields,
                metric_value,
            });
        }

        Ok(Self { file, parsed_data })
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct Row {
    #[serde(flatten)]
    simple_fields: SimpleFields,
    metric_value: Value,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct SimpleFields {
    #[serde(alias = "Category")]
    category: String,
    #[serde(alias = "Library Type")]
    library_type: String,
    #[serde(alias = "Grouped By")]
    grouped_by: String,
    #[serde(alias = "Group Name")]
    group_name: String,
    #[serde(alias = "Metric Name")]
    metric_name: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serde_json::{Number, Value};

    use super::{MultiRowCsv, Row, SimpleFields};

    #[rstest]
    fn parse_cellranger_multi_csv() {
        let raw_contents = include_str!("test-data/cellranger_multi.csv");
        let json = serde_json::json!({"filename": "file", "raw_contents": raw_contents});
        let MultiRowCsv {
            file: _,
            parsed_data,
        } = serde_json::from_value(json).unwrap();

        // The first row is easy
        let first_row = &parsed_data[0];
        assert_eq!(
            first_row,
            &Row {
                simple_fields: SimpleFields {
                    category: "Cells".to_owned(),
                    library_type: "Gene Expression".to_owned(),
                    grouped_by: String::new(),
                    group_name: String::new(),
                    metric_name: "Cells".to_owned()
                },
                metric_value: Value::Number(Number::from(1_866))
            },
        );

        // The second row is a percentage
        assert_eq!(parsed_data[1].metric_value.as_f64().unwrap(), 0.9314);

        // The 13th row requires extraction
        assert_eq!(parsed_data[12].metric_value.as_i64().unwrap(), 13_640);
    }
}
