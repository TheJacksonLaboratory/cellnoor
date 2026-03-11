#[cfg(feature = "app")]
use cellnoor_schema::institutions;
use macro_attributes::select;
use uuid::Uuid;

use crate::institution::common::InstitutionFields;

#[select]
pub struct Institution {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: InstitutionFields,
    #[cfg_attr(feature = "app", diesel(embed))]
    links: InstitutionLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = institutions))]
pub struct InstitutionLinks {
    members: String,
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
