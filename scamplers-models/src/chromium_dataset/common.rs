use macro_attributes::insert_select;
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::chromium_datasets;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetFields {
    name: NonEmptyString,
    lab_id: Uuid,
    data_path: NonEmptyString,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Eq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
pub(super) struct MetricsFile {
    pub(super) filename: NonEmptyString,
    pub(super) raw_contents: String,
}
