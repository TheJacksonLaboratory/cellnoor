#[cfg(feature = "builder")]
use bon::bon;
use macro_attributes::insert;
#[cfg(feature = "builder")]
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::institutions;
#[cfg(feature = "builder")]
use uuid::Uuid;

use crate::institution::common::InstitutionFields;

#[insert]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = institutions))]
pub struct InstitutionCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: InstitutionFields,
}

#[cfg_attr(feature = "builder", bon)]
impl InstitutionCreation {
    #[cfg(feature = "builder")]
    #[builder(on(_, into))]
    #[must_use]
    pub fn new(id: Uuid, name: NonEmptyString) -> Self {
        Self {
            inner: InstitutionFields { id, name },
        }
    }
}
