use macro_attributes::select;

use crate::{Relation, nonempty::NonemptyString, suspension_pool::MultiplexingTagType};

#[select]
#[derive(Eq, Hash)]
#[cfg_attr(feature = "postgres-types", postgres(name = "multiplexing_tag"))]
pub struct MultiplexingTag {
    pub tag_id: NonemptyString,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    #[cfg_attr(feature = "postgres-types", postgres(name = "type"))]
    pub type_: MultiplexingTagType,
}

impl Relation for MultiplexingTag {
    const NAME: &'static str = "multiplexing_tag";
}

pub type NewMultiplexingTag = MultiplexingTag;
