#[cfg(feature = "app")]
use cellnoor_schema::projects;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = projects))]
pub struct ProjectFields {
    pub(super) name: NonEmptyString,
}
