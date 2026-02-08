#[cfg(feature = "app")]
use cellnoor_schema::people;
use macro_attributes::update;
use non_empty::NonEmptyString;
use uuid::Uuid;

#[update]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
#[cfg_attr(feature = "builder", builder(on(_, into)))]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonUpdate {
    #[serde(skip)]
    #[cfg_attr(feature = "builder", builder(skip))]
    id: Uuid,
    name: Option<NonEmptyString>,
    email: Option<NonEmptyString>,
    microsoft_entra_oid: Option<Uuid>,
    orcid: Option<NonEmptyString>,
    institution_id: Option<Uuid>,
    is_admin: Option<bool>,
    is_biology_staff: Option<bool>,
    is_computational_staff: Option<bool>,
}
impl PersonUpdate {
    pub fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_ref().map(NonEmptyString::as_ref)
    }
}
