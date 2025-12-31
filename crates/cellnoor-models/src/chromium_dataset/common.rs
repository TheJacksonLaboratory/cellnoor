#[cfg(feature = "app")]
use cellnoor_schema::chromium_datasets;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetFields {
    pub(super) name: NonEmptyString,
    pub(super) lab_id: Uuid,
}
