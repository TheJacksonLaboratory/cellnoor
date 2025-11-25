use macro_attributes::insert_select;
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::labs;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct LabFields {
    pub(super) name: NonEmptyString,
    pub(super) pi_id: Uuid,

    pub(super) delivery_dir: NonEmptyString,
}
