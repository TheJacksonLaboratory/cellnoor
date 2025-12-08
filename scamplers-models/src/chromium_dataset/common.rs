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
