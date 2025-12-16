use macro_attributes::insert_select;
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::chromium_datasets;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetFields {
    pub(super) name: NonEmptyString,
    pub(super) lab_id: Uuid,
    pub(super) data_path: NonEmptyString,
}
