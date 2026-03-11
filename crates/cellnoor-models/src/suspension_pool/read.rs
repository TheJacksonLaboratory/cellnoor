#[cfg(feature = "app")]
use cellnoor_schema::suspension_pools;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::suspension_pool::SuspensionPoolFields;

#[select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_pools))]
pub struct SuspensionPool {
    id: Uuid,
    project_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionPoolFields,
    multiplexing_type: String,
    #[cfg_attr(feature = "app", diesel(embed))]
    links: SuspensionPoolLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = suspension_pools))]
pub struct SuspensionPoolLinks {
    measurements: String,
    suspensions: String,
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

    #[must_use]
    pub fn project_id(&self) -> Uuid {
        self.project_id
    }
}
