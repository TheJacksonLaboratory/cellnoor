use macro_attributes::insert_select;
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::libraries;
use serde_json::Value;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = libraries))]
pub struct LibraryFields {
    pub(super) readable_id: NonEmptyString,
    pub(super) cdna_id: Uuid,
    pub(super) single_index_set_name: Option<NonEmptyString>,
    pub(super) dual_index_set_name: Option<NonEmptyString>,
    pub(super) additional_data: Option<Value>,
}
