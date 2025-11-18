use macro_attributes::insert_select;
use non_empty_string::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::labs;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct Fields {
    #[cfg_attr(feature = "typescript", ts(inline))]
    pub(super) name: NonEmptyString,
    pub(super) pi_id: Uuid,
    #[cfg_attr(feature = "typescript", ts(inline))]
    pub(super) delivery_dir: NonEmptyString,
}
