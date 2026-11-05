use macro_attributes::{base_model, select};
pub use query::{PersonField, PersonPredicate, PersonQuery, SimplePersonQuery};
use uuid::Uuid;

use crate::{
    Relation, nonempty::NonemptyString, permission::Permission, simple_links::SimpleLinks,
};

mod query;

#[base_model]
pub struct PersonSimpleFields {
    pub name: NonemptyString,
    pub email: Option<NonemptyString>,
    pub institution_id: Uuid,
    pub is_staff: bool,
    pub orcid: Option<NonemptyString>,
}

#[base_model]
#[derive(Copy, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, tag = "auth_provider", rename_all = "snake_case")
)]
pub enum AuthAccount {
    Microsoft { microsoft_entra_oid: Uuid },
}

#[base_model]
pub struct NewPerson {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: PersonSimpleFields,
    pub auth_account: Option<AuthAccount>,
    pub permissions_to_grant: Vec<Permission>,
}

pub type PersonUpdate = PersonSimpleFields;

impl Relation for NewPerson {
    const NAME: &'static str = "person";
}

impl Relation for PersonUpdate {
    const NAME: &'static str = "person";
}

#[base_model]
pub struct PersonLinks {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SimpleLinks,
    pub projects: String,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "person_public"))]
pub struct SavedPersonRecord {
    pub id: Uuid,
    pub name: NonemptyString,
    pub email: Option<NonemptyString>,
    pub institution_id: Uuid,
    pub is_staff: bool,
    pub orcid: Option<NonemptyString>,
}

impl Relation for SavedPersonRecord {
    const NAME: &'static str = "person_public";
}

#[base_model]
pub struct Person {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedPersonRecord,
    pub links: PersonLinks,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "person_with_permissions"))]
pub struct PersonWithPermissions {
    #[cfg_attr(feature = "serde", serde(flatten))]
    #[cfg_attr(feature = "postgres-types", postgres(name = "person_public"))]
    pub person: SavedPersonRecord,
    pub permissions: Vec<Permission>,
}
