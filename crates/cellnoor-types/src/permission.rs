use macro_attributes::{select, unit_enum};

#[unit_enum]
#[derive(strum::VariantArray)]
pub enum Action {
    Create,
    Update,
    Delete,
}

#[unit_enum]
#[derive(strum::VariantArray)]
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
