use macro_attributes::insert_select;
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::cdna;
use serde_json::Value;
use uuid::Uuid;

use crate::tenx_assay::LibraryType;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = cdna))]
pub struct CdnaFields {
    pub(super) library_type: LibraryType,
    pub(super) readable_id: NonEmptyString,
    pub(super) gem_pool_id: Option<Uuid>,
    pub(super) additional_data: Option<Value>,
}
