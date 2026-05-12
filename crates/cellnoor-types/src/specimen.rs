pub use creation::{
    NewSpecimen,
    block::{BlockEmbeddingMatrix, BlockFixative, NewBlock},
    suspension::{NewSuspensionSpecimen, SuspensionThermalPreservation},
    tissue::NewTissue,
};
use jiff::Timestamp;
use macro_attributes::{base_model, select, unit_enum};
use nonempty::NonemptyString;
pub use query::{SimpleSpecimenQuery, SpecimenPredicate, SpecimenQuery};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    id::{Id, NoId},
    project::{Project, SavedProject},
    simple_links::SimpleLinks,
    specimen::{
        measurement::{NewSpecimenMeasurement, SpecimenMeasurement},
        record::{Species, SpecimenRecord},
    },
};

mod creation;
pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::{select, unit_enum};
    use nonempty::NonemptyString;
    use uuid::Uuid;

    use crate::specimen::BlockEmbeddingMatrix;

    #[select]
    #[cfg_attr(feature = "schemars", schemars(inline))]
    #[cfg_attr(feature = "postgres-types", postgres(name = "specimen"))]
    pub struct SpecimenRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
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
        pub additional_data: Option<serde_json::Value>,
        pub type_: SpecimenType,
        pub embedded_in: Option<BlockEmbeddingMatrix>,
        pub fixative: Option<Fixative>,
        pub thermal_preservation_method: Option<ThermalPreservationMethod>,
    }

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

pub type SavedSpecimen = SpecimenRecord<Id>;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_detailed"))]
pub struct SavedSpecimenDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub specimen: SavedSpecimen,
    pub project: SavedProject,
    pub measurements: Vec<SpecimenMeasurement>,
}

#[base_model]
pub enum Specimen {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedSpecimen,
        links: SimpleLinks,
    },
    // Rather than just wrapping the `SpecimenRecordDetailed`, we destructure its fields so that
    // we have a `Project` rather than a `ProjectRecord`
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedSpecimen,
        links: SimpleLinks,
        project: Project,
        measurements: Vec<SpecimenMeasurement>,
    },
}

impl SimpleLinks {
    fn for_specimen(id: Uuid) -> Self {
        Self::from_str_and_id("/specimens", id)
    }
}

impl Specimen {
    pub fn record(&self) -> &SavedSpecimen {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed { record, .. } => record,
        }
    }

    pub fn from_record(record: SavedSpecimen) -> Self {
        Self::Compact {
            links: SimpleLinks::for_specimen(*record.id),
            record,
        }
    }

    pub fn from_detailed_record(
        SavedSpecimenDetailed {
            specimen,
            project,
            measurements,
        }: SavedSpecimenDetailed,
    ) -> Self {
        Self::Detailed {
            links: SimpleLinks::for_specimen(*specimen.id),
            record: specimen,
            project: Project::from_record(project),
            measurements,
        }
    }
}
