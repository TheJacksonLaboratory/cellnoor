use jiff::Timestamp;
use macro_attributes::base_model;
use nonempty::NonemptyString;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    id::NoId,
    specimen::{
        Fixative, NewSpecimenRecord, Species, SpecimenType, ThermalPreservationMethod,
        creation::{
            block::{BlockEmbeddingMatrix, NewBlock},
            cell_pellet::NewCellPellet,
            rna_extract::NewRnaExtract,
            suspension::NewSuspensionSpecimen,
            tissue::NewTissue,
        },
        measurement::NewSpecimenMeasurement,
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

// We have to repeat these fields because only a subset are common to all
// specimen types, and there's no attribute like `postgres(flatten)`
#[base_model]
pub struct NewSpecimenCommonFields {
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub submitted_by: Uuid,
    pub received_at: Timestamp,
    pub project_id: Uuid,
    pub species: Species,
    pub host_species: Option<Species>,
    pub returned_by: Option<Uuid>,
    pub returned_at: Option<Timestamp>,
    pub tissue: NonemptyString,
    pub additional_data: Option<Value>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub measurements: Vec<NewSpecimenMeasurement>,
}

impl NewSpecimen {
    pub fn common_mut(&mut self) -> &mut NewSpecimenCommonFields {
        use NewSpecimen::{Block, CellPellet, RnaExtract, Suspension, Tissue};

        match self {
            Block(
                NewBlock::CarboxymethylCellulose { common, .. }
                | NewBlock::OptimalCuttingTemperatureCompound { common, .. }
                | NewBlock::Paraffin { common, .. },
            )
            | CellPellet(NewCellPellet { common, .. })
            | RnaExtract(NewRnaExtract { common, .. })
            | Suspension(
                NewSuspensionSpecimen::Fixed { common, .. }
                | NewSuspensionSpecimen::Fresh(common)
                | NewSuspensionSpecimen::ThermallyPreserved { common, .. },
            )
            | Tissue(
                NewTissue::Fixed { common, .. }
                | NewTissue::Fresh { common }
                | NewTissue::ThermallyPreserved { common, .. },
            ) => common,
        }
    }

    #[must_use]
    pub fn into_common(self) -> NewSpecimenCommonFields {
        use NewSpecimen::{Block, CellPellet, RnaExtract, Suspension, Tissue};

        match self {
            Block(
                NewBlock::CarboxymethylCellulose { common, .. }
                | NewBlock::OptimalCuttingTemperatureCompound { common, .. }
                | NewBlock::Paraffin { common, .. },
            )
            | CellPellet(NewCellPellet { common, .. })
            | RnaExtract(NewRnaExtract { common, .. })
            | Suspension(
                NewSuspensionSpecimen::Fixed { common, .. }
                | NewSuspensionSpecimen::Fresh(common)
                | NewSuspensionSpecimen::ThermallyPreserved { common, .. },
            )
            | Tissue(
                NewTissue::Fixed { common, .. }
                | NewTissue::Fresh { common }
                | NewTissue::ThermallyPreserved { common, .. },
            ) => common,
        }
    }

    #[must_use]
    pub fn split_for_insertion(self) -> (NewSpecimenRecord, Vec<NewSpecimenMeasurement>) {
        let SpecimenInsertion(record, measurements) = match self {
            Self::Block(s) => s.split_for_insertion(),
            Self::CellPellet(s) => s.split_for_insertion(),
            Self::RnaExtract(s) => s.split_for_insertion(),
            Self::Suspension(s) => s.split_for_insertion(),
            Self::Tissue(s) => s.split_for_insertion(),
        };

        (record, measurements)
    }
}

struct SpecimenInsertion(NewSpecimenRecord, Vec<NewSpecimenMeasurement>);

impl SpecimenInsertion {
    fn from_fields(
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
