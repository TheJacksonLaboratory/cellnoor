#[cfg(feature = "app")]
use cellnoor_schema::multiplexing_tags;
use macro_attributes::select;
use uuid::Uuid;

use crate::multiplexing_tag::common::MultiplexingTagFields;

#[select]
#[cfg_attr(feature = "app", diesel(table_name = multiplexing_tags))]
pub struct MultiplexingTag {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: MultiplexingTagFields,
}

impl MultiplexingTag {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}
