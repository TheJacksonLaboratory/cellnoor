use macro_attributes::insert_select;
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::sequencing_runs;
use serde_json::Value;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = sequencing_runs))]
pub struct SequencingRunFields {
    readable_id: NonEmptyString,
    additional_data: Option<Value>,
}
