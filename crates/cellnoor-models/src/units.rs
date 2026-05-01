use macro_attributes::unit_enum;
use macros::{impl_enum_from_sql, impl_enum_to_sql};

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[unit_enum]
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

#[unit_enum]
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

#[unit_enum]
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

#[unit_enum]
pub enum Picogram {
    #[serde(alias = "pg")]
    Picogram,
}

#[cfg(feature = "app")]
impl EnumFromSql for Picogram {}
impl_enum_from_sql!(Picogram);

#[cfg(feature = "app")]
impl EnumToSql for Picogram {}
impl_enum_to_sql!(Picogram);

#[unit_enum]
pub enum Nanogram {
    #[serde(alias = "ng")]
    Nanogram,
}

#[cfg(feature = "app")]
impl EnumFromSql for Nanogram {}
impl_enum_from_sql!(Nanogram);

#[cfg(feature = "app")]
impl EnumToSql for Nanogram {}
impl_enum_to_sql!(Nanogram);
