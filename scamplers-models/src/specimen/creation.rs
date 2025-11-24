use jiff::Timestamp;
use macro_attributes::base_model;

use crate::specimen::{
    common::{Species, SpecimenCommonFields},
    creation::block::{FixedBlockCreation, FrozenBlockCreation},
};

pub mod block;
pub mod suspension;
pub mod tissue;

#[base_model]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpecimenCreation {
    FixedBlock(FixedBlockCreation),
    FrozenBlock(FrozenBlockCreation),
}

impl SpecimenCreation {
    fn inner(&self) -> &SpecimenCommonFields {
        use SpecimenCreation::*;

        match self {
            FixedBlock(s) => &s.inner,
            FrozenBlock(s) => &s.inner,
        }
    }

    pub fn received_at(&self) -> Timestamp {
        self.inner().received_at
    }

    pub fn returned_at(&self) -> Option<Timestamp> {
        self.inner().returned_at
    }

    pub fn species(&self) -> Species {
        self.inner().species
    }

    pub fn host_species(&self) -> Option<Species> {
        self.inner().host_species
    }
}
