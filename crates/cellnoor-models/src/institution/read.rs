#[cfg(feature = "app")]
use cellnoor_schema::institutions;
use macro_attributes::select;
use uuid::Uuid;

use crate::institution::common::InstitutionFields;

#[select]
pub struct Institution {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: InstitutionFields,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: InstitutionLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = institutions))]
pub struct InstitutionLinks {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "members")]
    pub members_link: String,
}

impl Institution {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.inner.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name.as_ref()
    }
}
