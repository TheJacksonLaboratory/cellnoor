#[cfg(feature = "app")]
use cellnoor_schema::labs;
use macro_attributes::insert;
use non_empty::NonEmptyString;
use uuid::Uuid;

use crate::lab::common::LabFields;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct LabCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LabFields,
}
impl LabCreation {
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
