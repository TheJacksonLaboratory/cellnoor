#![allow(clippy::iter_without_into_iter)]
use std::slice::Iter;

use macro_attributes::{base_model, select, unit_enum};
use nonempty::NonemptyString;
pub use query::{PersonField, PersonPredicate, PersonQuery, SimplePersonQuery};
use uuid::Uuid;

use crate::simple_links::SimpleLinks;

mod query;

#[unit_enum]
pub enum Action {
    // For most insertions, we add a returning clause, which requires the select privilege
    #[strum(serialize = "select, insert")]
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[strum(serialize_all = "snake_case")]
pub enum ResourcePermission {
    Institution(Vec<Action>),
    Person(Vec<Action>),
    Account(Vec<Action>),
    Project(Vec<Action>),
    Specimen(Vec<Action>),
    AssayConstantData(Vec<Action>),
    ChromiumExperimentalData(Vec<Action>),
    ChromiumDataset(Vec<Action>),
}

#[base_model]
#[derive(Default)]
pub struct PermissionsToGrant(Vec<ResourcePermission>);

impl PermissionsToGrant {
    pub fn iter(&self) -> Iter<'_, ResourcePermission> {
        self.0.iter()
    }

    pub fn contains(&self, x: &ResourcePermission) -> bool {
        self.0.contains(x)
    }
}

impl From<Vec<ResourcePermission>> for PermissionsToGrant {
    fn from(value: Vec<ResourcePermission>) -> Self {
        Self(value)
    }
}

#[base_model]
#[derive(Default)]
pub struct PermissionsToRevoke(Vec<ResourcePermission>);

impl PermissionsToRevoke {
    pub fn iter(&self) -> Iter<'_, ResourcePermission> {
        self.0.iter()
    }

    pub fn contains(&self, x: &ResourcePermission) -> bool {
        self.0.contains(x)
    }
}

impl From<Vec<ResourcePermission>> for PermissionsToRevoke {
    fn from(value: Vec<ResourcePermission>) -> Self {
        Self(value)
    }
}

#[base_model]
pub struct PersonSimpleFields {
    pub name: NonemptyString,
    pub institution_id: Uuid,
    pub is_staff: bool,
    pub can_manage_users: bool,
    pub orcid: Option<NonemptyString>,
}

#[base_model]
#[derive(strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, tag = "auth_provider", rename_all = "snake_case")
)]
pub enum Account {
    Microsoft {
        microsoft_entra_oid: Uuid,
    },
    #[cfg_attr(feature = "serde", serde(untagged))]
    None {
        email: NonemptyString,
    },
}

#[base_model]
pub struct NewPerson {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: PersonSimpleFields,
    pub account: Account,
    pub permissions_to_grant: PermissionsToGrant,
}

#[base_model]
pub struct PersonUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: PersonSimpleFields,
    pub email: NonemptyString,
    pub permissions_to_grant: Option<PermissionsToGrant>,
    pub permissions_to_revoke: Option<PermissionsToRevoke>,
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
    pub can_manage_users: bool,
    pub orcid: Option<NonemptyString>,
}

#[base_model]
pub struct Person {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedPersonRecord,
    pub links: PersonLinks,
}
