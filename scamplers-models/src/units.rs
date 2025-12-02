use macro_attributes::simple_enum;
use macros::{impl_enum_from_sql, impl_enum_to_sql};

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[simple_enum]
pub enum Microliter {
    #[serde(alias = "µL")]
    Microliter,
}

#[cfg(feature = "app")]
impl EnumFromSql for Microliter {}
impl_enum_from_sql!(Microliter);

#[cfg(feature = "app")]
impl EnumToSql for Microliter {}
impl_enum_to_sql!(Microliter);

#[simple_enum]
pub enum Milliliter {
    #[serde(alias = "mL")]
    Milliliter,
}

#[cfg(feature = "app")]
impl EnumFromSql for Milliliter {}
impl_enum_from_sql!(Milliliter);

#[cfg(feature = "app")]
impl EnumToSql for Milliliter {}
impl_enum_to_sql!(Milliliter);

#[simple_enum]
pub enum Micrometer {
    #[serde(alias = "µm")]
    Micrometer,
}

#[cfg(feature = "app")]
impl EnumFromSql for Micrometer {}
impl_enum_from_sql!(Micrometer);

#[cfg(feature = "app")]
impl EnumToSql for Micrometer {}
impl_enum_to_sql!(Micrometer);
