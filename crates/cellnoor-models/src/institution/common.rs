#[cfg(feature = "app")]
use cellnoor_schema::institutions;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = institutions))]
pub struct InstitutionFields {
    pub(super) id: Uuid,
    pub(super) name: NonEmptyString,
}
