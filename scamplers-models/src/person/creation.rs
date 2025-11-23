#[cfg(feature = "builder")]
use bon::bon;
use macro_attributes::insert;
use non_empty_string::NonEmptyString;
#[cfg(feature = "app")]
use scamplers_schema::people;
use uuid::Uuid;

use crate::person::common::{Fields, UserRole};

#[insert]
#[cfg_attr(feature = "app", derive(diesel::AsChangeset))]
#[cfg_attr(feature = "app", diesel(table_name = people))]
#[cfg_attr(feature = "typescript", ts(rename = "PersonCreation"))]
pub struct Creation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: Fields,

    email: NonEmptyString,
    #[serde(default)]
    #[cfg_attr(feature = "app", diesel(skip_insertion, skip_update))]
    roles: Vec<UserRole>,
}

#[cfg_attr(feature = "builder", bon)]
impl Creation {
    #[cfg(feature = "builder")]
    #[builder(on(_, into))]
    #[must_use]
    pub fn new(
        name: NonEmptyString,
        email: NonEmptyString,
        orcid: Option<NonEmptyString>,
        institution_id: Uuid,
        microsoft_entra_oid: Option<Uuid>,
        roles: Vec<UserRole>,
    ) -> Self {
        Self {
            inner: Fields {
                name,
                orcid,
                institution_id,
                microsoft_entra_oid,
            },
            email,
            roles,
        }
    }

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
