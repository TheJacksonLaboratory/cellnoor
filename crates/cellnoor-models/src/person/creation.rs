#[cfg(feature = "app")]
use cellnoor_schema::people;
use macro_attributes::insert;
use non_empty::NonEmptyString;
use uuid::Uuid;

use crate::person::common::{PersonFields, UserRole};

#[insert]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: PersonFields,
    email: NonEmptyString,
    #[serde(default)]
    #[cfg_attr(feature = "app", diesel(skip_insertion, skip_update))]
    roles: Vec<UserRole>,
}

impl PersonCreation {
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
        self.inner.microsoft_entra_oid
    }

    #[must_use]
    pub fn email(&self) -> &str {
        self.email.as_ref()
    }

    #[must_use]
    pub fn roles(&self) -> &[UserRole] {
        &self.roles
    }

    pub fn roles_mut(&mut self) -> &mut Vec<UserRole> {
        &mut self.roles
    }
}
