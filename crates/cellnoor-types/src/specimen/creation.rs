use jiff::Timestamp;
use macro_attributes::base_model;
use uuid::Uuid;

use crate::specimen::{
    Species, SpecimenCommonFields, SpecimenVariableFields,
    creation::{
        block::NewBlock, cell_pellet::NewCellPellet, rna_extract::NewRnaExtract,
        suspension::NewSuspensionSpecimen, tissue::NewTissue,
    },
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

type SpecimenInsertion = (SpecimenCommonFields, SpecimenVariableFields);
