use std::collections::HashMap;

use heck::ToSnekCase;
use serde::{Deserialize, de};
use serde_json::Number;

use crate::chromium_dataset::{
    common::{MetricsFile, ParsedMetrics},
    creation::metrics::common::parse_str_as_number,
};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct SingleRowCsv {
    file: MetricsFile,
    parsed_data: HashMap<String, Number>,
}

impl From<SingleRowCsv> for ParsedMetrics {
    fn from(SingleRowCsv { file, parsed_data }: SingleRowCsv) -> Self {
        ParsedMetrics::SingleRowCsv { file, parsed_data }
    }
}

impl<'de> Deserialize<'de> for SingleRowCsv {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let file = MetricsFile::deserialize(deserializer)?;
        let mut csv = csv::Reader::from_reader(file.raw_contents.as_bytes());

        let header = csv.headers().map_err(de::Error::custom)?;
        let header_len = header.len();
        let snake_case_header: Vec<String> = header.iter().map(snake_case_field_name).collect();
        let mut records = csv.records();

        let n_rows_err = Err(de::Error::custom(
            "single-row CSV must have exactly one record",
        ));

        let first_record = match records.next().map(|r| r.map_err(de::Error::custom)) {
            Some(rec) => rec?,
            None => {
                return n_rows_err;
            }
        };

        if records.next().is_some() {
            return n_rows_err;
        }

        let mut parsed_data = HashMap::with_capacity(header_len);

        // Manual insertion into the map is preferred over `collect` because the latter
        // would require an extra iteration to transform `Vec<Result<_>>` to
        // `Result<Vec<_>>` before constructing the two-tuple
        for (field_name, field_value) in snake_case_header.into_iter().zip(first_record.iter()) {
            parsed_data.insert(
                field_name,
                parse_str_as_number(field_value).map_err(de::Error::custom)?,
            );
        }

        Ok(Self { file, parsed_data })
    }
}

fn snake_case_field_name(field_name: &str) -> String {
    let field_name = field_name.replace("UMIs", "umis");
    field_name.to_snek_case()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::SingleRowCsv;

    #[rstest]
    fn parse_single_row_csv() {
        let raw_contents = include_str!("test-data/single-row.csv");
        let json = serde_json::json!({"filename": "file", "raw_contents": raw_contents});
        let SingleRowCsv {
            file: _,
            parsed_data,
        } = serde_json::from_value(json).unwrap();

        assert_eq!(
            parsed_data["estimated_number_of_cells"].as_i64().unwrap(),
            65_558
        );

        assert_eq!(parsed_data["valid_barcodes"].as_f64().unwrap(), 0.95);

        assert!((parsed_data["valid_umis"].as_f64().unwrap() - 0.999).abs() < 1e-15);
    }
}
