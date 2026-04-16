#[cfg(feature = "app")]
use cellnoor_schema::people;
use macro_attributes::insert_select;
use non_empty::NonEmptyString;
use uuid::Uuid;

#[insert_select]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonFields {
    pub name: NonEmptyString,
    pub orcid: Option<NonEmptyString>,
    pub institution_id: Uuid,
}
