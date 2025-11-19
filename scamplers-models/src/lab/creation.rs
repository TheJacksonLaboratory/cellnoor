#[cfg(feature = "builder")]
use bon::bon;
use macro_attributes::insert;
#[cfg(feature = "builder")]
use non_empty_string::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::labs;
#[cfg(feature = "builder")]
use uuid::Uuid;

use crate::lab::common::Fields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
#[cfg_attr(feature = "typescript", ts(rename = "LabCreation"))]
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
    pub fn new(name: NonEmptyString, pi_id: Uuid, delivery_dir: NonEmptyString) -> Self {
        Self {
            inner: Fields {
                name,
                pi_id,
                delivery_dir,
            },
        }
    }

    #[must_use]
    pub fn delivery_dir(&self) -> &str {
        self.inner.delivery_dir.as_ref()
    }
}
