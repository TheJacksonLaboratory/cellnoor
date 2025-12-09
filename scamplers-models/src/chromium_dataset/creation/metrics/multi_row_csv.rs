use serde::{Deserialize, de};
use serde_json::Value;

use crate::{
    chromium_dataset::{common::MetricsFile, creation::metrics::common::parse_str_as_number},
    tenx_assay::LibraryType,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(optional_fields))]
pub struct MultiRowCsv {
    #[serde(flatten)]
    file: MetricsFile,
    #[cfg_attr(feature = "typescript", ts(skip))]
    parsed_data: Vec<Row>,
}

impl<'de> Deserialize<'de> for MultiRowCsv {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let file = MetricsFile::deserialize(deserializer)?;
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

#[derive(Clone, Debug, PartialEq, serde::Serialize, Eq)]
struct Row {
    #[serde(flatten)]
    simple_fields: SimpleFields,
    metric_value: Value,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
struct SimpleFields {
    #[serde(rename(deserialize = "Category"))]
    category: String,
    #[serde(rename(deserialize = "Library Type"))]
    library_type: LibraryType,
    #[serde(rename(deserialize = "Grouped By"))]
    grouped_by: String,
    #[serde(rename(deserialize = "Group Name"))]
    group_name: String,
    #[serde(rename(deserialize = "Metric Name"))]
    metric_name: String,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;
    use serde_json::{Number, Value};

    use super::{MultiRowCsv, Row, SimpleFields};
    use crate::tenx_assay::LibraryType;

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
                    library_type: LibraryType::GeneExpression,
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
