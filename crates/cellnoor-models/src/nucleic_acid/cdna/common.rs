#[cfg(feature = "app")]
use cellnoor_schema::cdna;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use serde_json::Value;
use uuid::Uuid;

use crate::tenx_assay::LibraryType;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = cdna))]
pub struct CdnaFields {
    pub library_type: LibraryType,
    pub readable_id: NonEmptyString,
    pub gem_pool_id: Option<Uuid>,
    pub additional_data: Option<Value>,
}
