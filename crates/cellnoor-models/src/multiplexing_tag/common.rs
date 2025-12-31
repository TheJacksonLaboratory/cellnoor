#[cfg(feature = "app")]
use cellnoor_schema::multiplexing_tags;
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::{insert_select, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use non_empty::NonEmptyString;

#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

#[insert_select]
#[cfg_attr(feature = "app", derive(AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = multiplexing_tags))]
pub struct MultiplexingTagFields {
    tag_id: NonEmptyString,
    type_: MultiplexingTagType,
}

#[simple_enum]
pub enum MultiplexingTagType {
    FlexBarcode,
    OnChipMultiplexing,
    #[serde(rename = "TotalSeq-A")]
    #[strum(serialize = "TotalSeq-A")]
    TotalSeqA,
    #[serde(rename = "TotalSeq-B")]
    #[strum(serialize = "TotalSeq-B")]
    TotalSeqB,
    #[serde(rename = "TotalSeq-C")]
    #[strum(serialize = "TotalSeq-C")]
    TotalSeqC,
}

#[cfg(feature = "app")]
impl EnumFromSql for MultiplexingTagType {}
impl_enum_from_sql!(MultiplexingTagType);

#[cfg(feature = "app")]
impl EnumToSql for MultiplexingTagType {}
impl_enum_to_sql!(MultiplexingTagType);
