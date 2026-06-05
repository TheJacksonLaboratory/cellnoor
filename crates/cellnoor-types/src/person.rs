#![allow(clippy::iter_without_into_iter)]
use std::slice::Iter;

use macro_attributes::{base_model, unit_enum};
pub use query::{PersonField, PersonPredicate, PersonQuery, SimplePersonQuery};

use crate::{
    id::{Id, NoId},
    person::record::PersonRecord,
    simple_links::SimpleLinks,
};

mod query;

mod record {
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use uuid::Uuid;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "person_public"))]
    pub struct PersonRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub name: NonemptyString,
        pub email: Option<NonemptyString>,
        pub institution_id: Uuid,
        #[cfg_attr(feature = "serde", serde(default))]
        pub is_staff: bool,
        pub orcid: Option<NonemptyString>,
    }
}

#[unit_enum]
pub enum Action {
    // For most insertions, we add a returning clause, which requires the select privilege
    #[strum(serialize = "select, insert")]
    Create,
    Update,
    Delete,
}

impl Action {
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ResourcePermission {
    Institution(Vec<Action>),
    Person(Vec<Action>),
    Project(Vec<Action>),
    Specimen(Vec<Action>),
    AssayConstantData(Vec<Action>),
    ChromiumExperimentalData(Vec<Action>),
    ChromiumDataset(Vec<Action>),
}

pub type NewPersonRecord = PersonRecord<NoId>;

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
pub struct NewPerson {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewPersonRecord,
    pub permissions_to_grant: PermissionsToGrant,
}

#[base_model]
pub struct PersonUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewPersonRecord,
    pub permissions_to_grant: Option<PermissionsToGrant>,
    pub permissions_to_revoke: Option<PermissionsToRevoke>,
}

#[base_model]
pub struct PersonLinks {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SimpleLinks,
    pub projects: String,
}

pub type SavedPersonRecord = PersonRecord<Id>;

#[base_model]
pub struct Person {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedPersonRecord,
    pub links: PersonLinks,
}
