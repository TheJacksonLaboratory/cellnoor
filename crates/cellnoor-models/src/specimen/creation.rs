use jiff::Timestamp;
use macro_attributes::base_model;
use uuid::Uuid;

use crate::specimen::{
    common::{Species, SpecimenCommonFields},
    creation::{block::NewBlock, suspension::NewSuspensionSpecimen, tissue::NewTissue},
};

pub mod block;
pub mod suspension;
pub mod tissue;

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewSpecimen {
    Block(NewBlock),
    Suspension(NewSuspensionSpecimen),
    Tissue(NewTissue),
}

impl NewSpecimen {
    fn inner(&self) -> &SpecimenCommonFields {
        use NewSpecimen::{Block, Suspension, Tissue};

        match self {
            Block(s) => s.common(),
            Suspension(s) => s.common(),
            Tissue(s) => s.common(),
        }
    }

    #[must_use]
    pub fn received_at(&self) -> Timestamp {
        self.inner().received_at
    }

    #[must_use]
    pub fn returned_at(&self) -> Option<Timestamp> {
        self.inner().returned_at
    }

    #[must_use]
    pub fn species(&self) -> Species {
        self.inner().species
    }

    #[must_use]
    pub fn host_species(&self) -> Option<Species> {
        self.inner().host_species
    }

    #[must_use]
    pub fn project_id(&self) -> Uuid {
        self.inner().project_id
    }
}
