use macro_attributes::{select, sql_enum};

#[sql_enum]
#[cfg_attr(feature = "postgres-types", postgres(name = "action"))]
pub enum Action {
    Create,
    Update,
    Delete,
}

#[sql_enum]
#[cfg_attr(feature = "postgres-types", postgres(name = "resource"))]
pub enum Resource {
    Institution,
    Person,
    Account,
    Project,
    Specimen,
    AssayConstantData,
    ChromiumExperimentalData,
    ChromiumDataset,
}

#[select]
#[derive(Copy, Eq)]
#[cfg_attr(feature = "postgres-types", postgres(name = "granted_permission"))]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
}
