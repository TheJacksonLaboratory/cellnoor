#[cfg(feature = "builder")]
use bon::bon;
use macro_attributes::insert;
#[cfg(feature = "builder")]
use non_empty::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::labs;
#[cfg(feature = "builder")]
use uuid::Uuid;

use crate::lab::common::LabFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct LabCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LabFields,
}

#[cfg_attr(feature = "builder", bon)]
impl LabCreation {
    #[cfg(feature = "builder")]
    #[builder(on(_, into))]
    #[must_use]
    pub fn new(name: NonEmptyString, pi_id: Uuid, delivery_dir: NonEmptyString) -> Self {
        Self {
            inner: LabFields {
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
