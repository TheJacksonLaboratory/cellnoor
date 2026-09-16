use macro_attributes::{base_model, unit_enum};

#[unit_enum]
pub enum Action {
    Create,
    Update,
    Delete,
}

/// A group of tables that a permission is granted on.
///
/// Which tables each one covers is defined in
/// /db/migrations/0031_data-rls.up.sql.
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

/// Permission to take one action on one resource, which is one row of the
/// `permission` table.
#[base_model]
#[derive(Copy, Eq)]
pub struct Permission {
    pub resource: Resource,
    pub action: Action,
}
