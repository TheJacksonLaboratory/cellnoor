use jiff::Timestamp;
use macro_attributes::base_model;
use uuid::Uuid;

use crate::specimen::{
    common::{Species, SpecimenCommonFields},
    creation::{
        block::NewBlock, cell_pellet::NewCellPellet, suspension::NewSuspensionSpecimen,
        tissue::NewTissue,
    },
    variable::SpecimenVariableFields,
};

pub mod block;
pub mod cell_pellet;
pub mod suspension;
pub mod tissue;

#[base_model]
#[derive(serde::Deserialize)]
#[cfg_attr(feature = "app", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NewSpecimen {
    Block(NewBlock),
    CellPellet(NewCellPellet),
    Suspension(NewSuspensionSpecimen),
    Tissue(NewTissue),
}

type SpecimenInsertion = (SpecimenCommonFields, SpecimenVariableFields);

impl NewSpecimen {
    fn inner(&self) -> &SpecimenCommonFields {
        use NewSpecimen::{Block, CellPellet, Suspension, Tissue};

        match self {
            Block(s) => s.common(),
            CellPellet(s) => s.common(),
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
