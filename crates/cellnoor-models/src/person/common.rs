#[cfg(feature = "app")]
use cellnoor_schema::people;
use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty::NonEmptyString;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
pub enum UserRole {
    AppAdmin,
    BiologyStaff,
    ComputationalStaff,
}

#[cfg(feature = "app")]
impl EnumFromSql for UserRole {}
impl_enum_from_sql!(UserRole);

#[cfg(feature = "app")]
impl EnumToSql for UserRole {}
impl_enum_to_sql!(UserRole);

#[insert_select]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonFields {
    pub(super) name: NonEmptyString,
    pub(super) orcid: Option<NonEmptyString>,
    pub(super) institution_id: Uuid,
    pub(super) microsoft_entra_oid: Option<Uuid>,
}
