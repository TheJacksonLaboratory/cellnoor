use jiff::Timestamp;
use macro_attributes::base_model;

use crate::specimen::{
    common::{Species, SpecimenCommonFields},
    creation::{
        block::{FixedBlockCreation, FrozenBlockCreation},
        suspension::{
            CryopreservedSuspensionCreation, FixedSuspensionCreation, FreshSuspensionCreation,
            FrozenSuspensionCreation,
        },
        tissue::{CryopreservedTissueCreation, FixedTissueCreation, FrozenTissueCreation},
    },
};

pub mod block;
pub mod suspension;
pub mod tissue;

#[base_model]
#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpecimenCreation {
    FixedBlock(FixedBlockCreation),
    FrozenBlock(FrozenBlockCreation),
    CryopreservedSuspension(CryopreservedSuspensionCreation),
    FixedSuspension(FixedSuspensionCreation),
    FreshSuspension(FreshSuspensionCreation),
    FrozenSuspension(FrozenSuspensionCreation),
    CryopreservedTissue(CryopreservedTissueCreation),
    FixedTissue(FixedTissueCreation),
    FrozenTissue(FrozenTissueCreation),
}

impl SpecimenCreation {
    fn inner(&self) -> &SpecimenCommonFields {
        use SpecimenCreation::{
            CryopreservedSuspension, CryopreservedTissue, FixedBlock, FixedSuspension, FixedTissue,
            FreshSuspension, FrozenBlock, FrozenSuspension, FrozenTissue,
        };

        match self {
            FixedBlock(s) => &s.inner,
            FrozenBlock(s) => &s.inner,
            CryopreservedSuspension(s) => &s.inner,
            FixedSuspension(s) => &s.inner,
            FreshSuspension(s) => &s.inner,
            FrozenSuspension(s) => &s.inner,
            CryopreservedTissue(s) => &s.inner,
            FixedTissue(s) => &s.inner,
            FrozenTissue(s) => &s.inner,
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
