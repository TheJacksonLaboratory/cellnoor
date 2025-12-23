use serde_json::Value;

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

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub struct Row {
    #[serde(flatten)]
    simple_fields: SimpleFields,
    metric_value: Value,
}

impl Row {
    #[must_use]
    pub fn new(simple_fields: SimpleFields, metric_value: Value) -> Self {
        Self {
            simple_fields,
            metric_value,
        }
    }
}
