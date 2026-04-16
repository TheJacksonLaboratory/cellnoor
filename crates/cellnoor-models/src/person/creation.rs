#[cfg(feature = "app")]
use cellnoor_schema::people;
use macro_attributes::insert;
use non_empty::NonEmptyString;
use uuid::Uuid;

use crate::person::common::PersonFields;

#[insert]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct NewPerson {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: PersonFields,
    pub email: NonEmptyString,
    pub microsoft_entra_oid: Option<Uuid>,
}

impl NewPerson {
    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name.as_ref()
    }

    pub fn orcid(&self) -> Option<&str> {
        self.inner.orcid.as_ref().map(NonEmptyString::as_ref)
    }

    #[must_use]
    pub fn institution_id(&self) -> Uuid {
        self.inner.institution_id
    }

    #[must_use]
    pub fn microsoft_entra_oid(&self) -> Option<Uuid> {
        self.microsoft_entra_oid
    }

    #[must_use]
    pub fn email(&self) -> &str {
        self.email.as_ref()
    }
}
