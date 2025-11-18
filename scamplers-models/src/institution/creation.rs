#[cfg(feature = "builder")]
use bon::bon;
use macro_attributes::insert;
#[cfg(feature = "builder")]
use non_empty_string::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::institutions;
#[cfg(feature = "builder")]
use uuid::Uuid;

use crate::institution::common::Fields;

#[insert]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = institutions))]
#[cfg_attr(feature = "typescript", ts(rename = "InstitutionCreation"))]
pub struct Creation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: Fields,
}

#[cfg_attr(feature = "builder", bon)]
impl Creation {
    #[cfg(feature = "builder")]
    #[builder(on(_, into))]
    #[must_use]
    fn new(id: Uuid, name: NonEmptyString) -> Self {
        Self {
            inner: Fields { id, name },
        }
    }
}
