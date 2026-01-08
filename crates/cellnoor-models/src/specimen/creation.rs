use jiff::Timestamp;
use macro_attributes::base_model;

use crate::specimen::{
    common::{Species, SpecimenCommonFields},
    creation::{
        block::BlockCreation, suspension::SuspensionSpecimenCreation, tissue::TissueCreation,
    },
};

pub mod block;
pub mod suspension;
pub mod tissue;

#[base_model]
#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpecimenCreation {
    Block(BlockCreation),
    Suspension(SuspensionSpecimenCreation),
    Tissue(TissueCreation),
}

impl SpecimenCreation {
    fn inner(&self) -> &SpecimenCommonFields {
        use SpecimenCreation::{Block, Suspension, Tissue};

        match self {
            Block(s) => s.inner(),
            Suspension(s) => s.inner(),
            Tissue(s) => s.inner(),
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
}
