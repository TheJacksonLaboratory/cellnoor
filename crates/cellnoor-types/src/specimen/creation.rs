use macro_attributes::base_model;

use crate::specimen::{
    NewSpecimenCommonFields, NewSpecimenVariableFields,
    creation::{
        block::NewBlock, cell_pellet::NewCellPellet, rna_extract::NewRnaExtract,
        suspension::NewSuspensionSpecimen, tissue::NewTissue,
    },
    measurement::NewSpecimenMeasurement,
};

pub mod block;
pub mod cell_pellet;
pub mod rna_extract;
pub mod suspension;
pub mod tissue;

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum NewSpecimen {
    Block(NewBlock),
    CellPellet(NewCellPellet),
    RnaExtract(NewRnaExtract),
    Suspension(NewSuspensionSpecimen),
    Tissue(NewTissue),
}

pub type SpecimenInsertion = (
    (NewSpecimenCommonFields, Vec<NewSpecimenMeasurement>),
    NewSpecimenVariableFields,
);

impl NewSpecimen {
    pub fn split_for_insertion(self) -> SpecimenInsertion {
        match self {
            Self::Block(s) => s.split_for_insertion(),
            Self::CellPellet(s) => s.split_for_insertion(),
            Self::RnaExtract(s) => s.split_for_insertion(),
            Self::Suspension(s) => s.split_for_insertion(),
            Self::Tissue(s) => s.split_for_insertion(),
        }
    }
}
