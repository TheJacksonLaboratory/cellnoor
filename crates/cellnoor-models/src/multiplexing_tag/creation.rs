#[cfg(feature = "app")]
use cellnoor_schema::multiplexing_tags;
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::insert;

use crate::multiplexing_tag::common::MultiplexingTagFields;

#[insert]
#[cfg_attr(feature = "app", derive(AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = multiplexing_tags))]
pub struct MultiplexingTagCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: MultiplexingTagFields,
}
