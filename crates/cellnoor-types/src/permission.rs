use macro_attributes::{base_model, unit_enum};

#[unit_enum]
pub enum Action {
    Create,
    Update,
    Delete,
}

#[unit_enum]
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

#[base_model]
#[derive(Copy, Eq)]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
}
