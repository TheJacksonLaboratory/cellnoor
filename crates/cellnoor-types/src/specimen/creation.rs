use macro_attributes::base_model;

use crate::{
    id::NoId,
    specimen::{
        BlockEmbeddingMatrix, NewSpecimenCommonFields,
        creation::{
            block::NewBlock, cell_pellet::NewCellPellet, rna_extract::NewRnaExtract,
            suspension::NewSuspensionSpecimen, tissue::NewTissue,
        },
        measurement::NewSpecimenMeasurement,
        record::{Fixative, SpecimenRecord, SpecimenType, ThermalPreservationMethod},
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

pub type NewSpecimenRecord = SpecimenRecord<NoId>;

struct SpecimenInsertion(NewSpecimenRecord, Vec<NewSpecimenMeasurement>);

impl SpecimenInsertion {
    fn from_common_and_variable(
        NewSpecimenCommonFields {
            readable_id,
            name,
            submitted_by,
            received_at,
            project_id,
            species,
            host_species,
            returned_by,
            returned_at,
            tissue,
            additional_data,
            measurements,
        }: NewSpecimenCommonFields,
        type_: SpecimenType,
        embedded_in: Option<BlockEmbeddingMatrix>,
        fixative: Option<impl Into<Fixative>>,
        thermal_preservation_method: Option<impl Into<ThermalPreservationMethod>>,
    ) -> Self {
        Self(
            NewSpecimenRecord {
                id: NoId {},
                readable_id,
                name,
                submitted_by,
                received_at,
                project_id,
                species,
                host_species,
                returned_by,
                returned_at,
                tissue,
                additional_data,
                type_,
                embedded_in,
                fixative: fixative.map(Into::into),
                thermal_preservation_method: thermal_preservation_method.map(Into::into),
            },
            measurements,
        )
    }
}

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
