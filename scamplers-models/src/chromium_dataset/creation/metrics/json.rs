use std::collections::HashMap;

use serde::{Deserialize, de};
use serde_json::Value;

use crate::chromium_dataset::common::{ParsedMetrics, ParsedMetricsFile, RawMetricsFile};

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct Json {
    file: RawMetricsFile,
    parsed_data: HashMap<String, Value>,
}

impl From<Json> for ParsedMetrics {
    fn from(Json { file, parsed_data }: Json) -> Self {
        ParsedMetrics::Json(vec![ParsedMetricsFile { file, parsed_data }])
    }
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let file = RawMetricsFile::deserialize(deserializer)?;
        let parsed_data = serde_json::from_str(&file.raw_contents).map_err(de::Error::custom)?;

        Ok(Self { file, parsed_data })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::Json;

    #[rstest]
    fn parse_single_row_csv() {
        let raw_contents = include_str!("test-data/cellranger-atac_count.json");
        let json = serde_json::json!({"filename": "file", "raw_contents": raw_contents});
        let Json {
            file: _,
            parsed_data,
        } = serde_json::from_value(json).unwrap();

        assert_eq!(parsed_data["annotated_cells"].as_i64().unwrap(), 5725);
    }
}
