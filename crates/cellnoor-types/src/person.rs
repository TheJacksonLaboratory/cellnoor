use macro_attributes::{base_model, select, unit_enum};
use nonempty::NonemptyString;
use uuid::Uuid;

use crate::simple_links::SimpleLinks;

#[unit_enum]
pub enum Action {
    #[strum(serialize = "insert")]
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, ::strum::AsRefStr)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
#[strum(serialize_all = "snake_case")]
pub enum ResourcePermission {
    Institution(Vec<Action>),
    Person(Vec<Action>),
    Project(Vec<Action>),
    Specimen(Vec<Action>),
    ChromiumExperimentalEntities(Vec<Action>),
    ChromiumDataset(Vec<Action>),
}

#[base_model]
pub struct NewPerson {
    pub name: NonemptyString,
    pub institution_id: Uuid,
    pub email: NonemptyString,
    pub orcid: Option<NonemptyString>,
    pub is_staff: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub grant_permissions: Vec<ResourcePermission>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub revoke_permissions: Vec<ResourcePermission>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "person_public"))]
pub struct PersonRecord {
    pub name: String,
    pub institution_id: String,
    pub email: Option<String>,
    pub orcid: Option<String>,
}

#[base_model]
pub struct PersonLinks {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SimpleLinks,
    pub specimens: String,
    pub chromium_datasets: String,
    pub projects: String,
}

#[base_model]
pub struct Person {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: PersonRecord,
    pub links: PersonLinks,
}
