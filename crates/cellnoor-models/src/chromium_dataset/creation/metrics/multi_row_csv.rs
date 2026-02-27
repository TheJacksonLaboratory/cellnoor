use serde_json::Value;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
pub struct SimpleFields {
    #[serde(alias = "Category")]
    pub category: String,
    #[serde(alias = "Library Type")]
    pub library_type: String,
    #[serde(alias = "Grouped By")]
    pub grouped_by: String,
    #[serde(alias = "Group Name")]
    pub group_name: String,
    #[serde(alias = "Metric Name")]
    pub metric_name: String,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "app", derive(::schemars::JsonSchema))]
pub struct Row {
    #[serde(flatten)]
    pub simple_fields: SimpleFields,
    pub metric_value: Value,
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
