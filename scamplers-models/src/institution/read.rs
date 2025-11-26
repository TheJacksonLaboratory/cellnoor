use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::institutions;
use uuid::Uuid;

use crate::{institution::common::InstitutionFields, links::Links};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = institutions))]
pub struct Institution {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: InstitutionFields,
    links: Links,
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
