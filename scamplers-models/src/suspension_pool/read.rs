use jiff::Timestamp;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::suspension_pools;
use uuid::Uuid;

use crate::{links::Links, suspension_pool::SuspensionPoolFields};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_pools))]
pub struct SuspensionPool {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionPoolFields,
    links: Links,
}
impl SuspensionPool {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn pooled_at(&self) -> Timestamp {
        self.inner.pooled_at()
    }
}
