#[cfg(feature = "app")]
use cellnoor_schema::institutions;
use macro_attributes::insert;
use non_empty::NonEmptyString;
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

impl InstitutionCreation {
    #[must_use]
    pub fn new(id: Uuid, name: NonEmptyString) -> Self {
        Self {
            inner: InstitutionFields { id, name },
        }
    }
}
