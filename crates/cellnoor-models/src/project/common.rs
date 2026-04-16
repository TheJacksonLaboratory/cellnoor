#[cfg(feature = "app")]
use cellnoor_schema::projects;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;

#[insert_select]
#[cfg_attr(feature = "app", diesel(table_name = projects))]
pub struct ProjectFields {
    pub name: NonEmptyString,
}
