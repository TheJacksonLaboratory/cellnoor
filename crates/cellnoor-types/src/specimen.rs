pub use creation::{
    NewSpecimen,
    block::{BlockEmbeddingMatrix, BlockFixative, NewBlock},
    suspension::{NewSuspensionSpecimen, SuspensionThermalPreservation},
    tissue::NewTissue,
};
use jiff::Timestamp;
use macro_attributes::{base_model, select, unit_enum};
use nonempty::NonemptyString;
pub use query::{SpecimenPredicate, SpecimenQuery, SpecimenSortField};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    SimpleLinks,
    project::{Project, ProjectRecord},
    specimen::measurement::{NewSpecimenMeasurement, SpecimenMeasurement},
};

mod creation;
pub mod measurement;
mod query;

#[unit_enum]
pub enum Species {
    AmbystomaMexicanum,
    CanisFamiliaris,
    CallithrixJacchus,
    DrosophilaMelanogaster,
    GasterosteusAculeatus,
    HomoSapiens,
    MusMusculus,
    RattusNorvegicus,
    SminthopsisCrassicaudata,
}

#[base_model]
pub struct SpecimenCommonFields {
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

impl SpecimenCommonFields {
    fn split_for_insertion(mut self) -> (Self, Vec<NewSpecimenMeasurement>) {
        let measurements = self.measurements.drain(..).collect();

        (self, measurements)
    }
}

#[unit_enum]
pub enum SpecimenType {
    Block,
    CellPellet,
    RnaExtract,
    Suspension,
    Tissue,
}

#[unit_enum]
pub enum Fixative {
    DithiobisSuccinimidylpropionate,
    FormaldehydeDerivative,
}

#[unit_enum]
pub enum ThermalPreservationMethod {
    ControlledRateFreezing,
    FlashFreezing,
}

#[base_model]
pub struct SpecimenVariableFields {
    pub type_: SpecimenType,
    pub embedded_in: Option<BlockEmbeddingMatrix>,
    pub fixative: Option<Fixative>,
    pub thermal_preservation_method: Option<ThermalPreservationMethod>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen"))]
pub struct SpecimenRecord {
    pub id: Uuid,
    pub readable_id: NonemptyString,
    pub name: NonemptyString,
    pub submitted_by: Uuid,
    pub project_id: Uuid,
    pub received_at: Timestamp,
    pub species: Species,
    pub host_species: Option<Species>,
    pub returned_at: Option<Timestamp>,
    pub returned_by: Option<Uuid>,
    #[cfg_attr(feature = "postgres-types", postgres(name = "type"))]
    pub type_: SpecimenType,
    pub embedded_in: Option<BlockEmbeddingMatrix>,
    pub fixative: Option<Fixative>,
    pub thermal_preservation_method: Option<ThermalPreservationMethod>,
    pub tissue: NonemptyString,
    pub additional_data: Option<Value>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_detailed"))]
pub struct SpecimenRecordDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub specimen: SpecimenRecord,
    pub project: ProjectRecord,
    pub measurements: Vec<SpecimenMeasurement>,
}

#[base_model]
pub enum Specimen {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SpecimenRecord,
        links: SimpleLinks,
    },
    // Rather than just wrapping the `SpecimenRecordDetailed`, we destructure its fields so that
    // we have a `Project` rather than a `ProjectRecord`
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SpecimenRecord,
        links: SimpleLinks,
        project: Project,
        measurements: Vec<SpecimenMeasurement>,
    },
}

fn specimen_links(id: Uuid) -> SimpleLinks {
    SimpleLinks::from_str_and_id("/specimens", id)
}

impl Specimen {
    pub fn record(&self) -> &SpecimenRecord {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed { record, .. } => record,
        }
    }

    pub fn from_record(record: SpecimenRecord) -> Self {
        Self::Compact {
            links: specimen_links(record.id),
            record,
        }
    }

    pub fn from_detailed_record(
        SpecimenRecordDetailed {
            specimen,
            project,
            measurements,
        }: SpecimenRecordDetailed,
    ) -> Self {
        Self::Detailed {
            links: specimen_links(specimen.id),
            record: specimen,
            project: Project::from_record(project),
            measurements,
        }
    }
}
