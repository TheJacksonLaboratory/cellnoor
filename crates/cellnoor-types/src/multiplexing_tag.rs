use macro_attributes::select;
use nonempty::NonemptyString;

use crate::suspension_pool::MultiplexingTagType;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "multiplexing_tag"))]
pub struct MultiplexingTag {
    pub tag_id: NonemptyString,
    #[cfg_attr(feature = "postgres-types", postgres(name = "type"))]
    pub type_: MultiplexingTagType,
}

pub type NewMultiplexingTag = MultiplexingTag;
