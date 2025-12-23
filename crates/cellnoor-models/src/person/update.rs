#[cfg(feature = "app")]
use cellnoor_schema::people;
use macro_attributes::update;
use non_empty::NonEmptyString;
use uuid::Uuid;

use crate::person::UserRole;

#[update]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
#[cfg_attr(feature = "builder", builder(on(_, into)))]
#[cfg_attr(feature = "app", diesel(table_name = people))]
#[cfg_attr(feature = "typescript", ts(rename = "PersonUpdate"))]
pub struct PersonUpdate {
    #[serde(skip)]
    #[cfg_attr(feature = "builder", builder(skip))]
    id: Uuid,
    name: Option<NonEmptyString>,
    email: Option<NonEmptyString>,
    microsoft_entra_oid: Option<Uuid>,
    orcid: Option<NonEmptyString>,
    institution_id: Option<Uuid>,
    #[cfg_attr(feature = "app", diesel(skip_update))]
    grant_roles: Option<Vec<UserRole>>,
    #[cfg_attr(feature = "app", diesel(skip_update))]
    revoke_roles: Option<Vec<UserRole>>,
}
impl PersonUpdate {
    pub fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_ref().map(NonEmptyString::as_ref)
    }

    #[must_use]
    pub fn grant_roles(&self) -> Option<&[UserRole]> {
        self.grant_roles.as_deref()
    }

    #[must_use]
    pub fn revoke_roles(&self) -> Option<&[UserRole]> {
        self.revoke_roles.as_deref()
    }
}
