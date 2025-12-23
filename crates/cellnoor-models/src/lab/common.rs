#[cfg(feature = "app")]
use cellnoor_schema::labs;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct LabFields {
    pub(super) name: NonEmptyString,
    pub(super) pi_id: Uuid,
    pub(super) delivery_dir: NonEmptyString,
}
