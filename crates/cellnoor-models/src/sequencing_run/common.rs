#[cfg(feature = "app")]
use cellnoor_schema::sequencing_runs;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use serde_json::Value;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_runs))]
pub struct SequencingRunFields {
    readable_id: NonEmptyString,
    additional_data: Option<Value>,
}
