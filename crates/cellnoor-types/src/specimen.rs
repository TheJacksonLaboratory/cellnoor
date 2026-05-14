use macro_attributes::{base_model, select, unit_enum};
pub use query::{SimpleSpecimenQuery, SpecimenPredicate, SpecimenQuery};

use crate::{
    id::Id,
    project::{Project, SavedProjectRecord},
    simple_links::SimpleLinks,
    specimen::{measurement::SpecimenMeasurement, record::SpecimenRecord},
};

pub mod creation;
pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use uuid::Uuid;

    use crate::specimen::{
        Fixative, Species, SpecimenType, ThermalPreservationMethod,
        creation::block::BlockEmbeddingMatrix,
    };

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "specimen"))]
    pub struct SpecimenRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
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
        pub additional_data: Option<serde_json::Value>,
    }
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

pub type SavedSpecimenRecord = SpecimenRecord<Id>;

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "specimen_detailed"))]
pub struct SavedSpecimenRecordDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub specimen: SavedSpecimenRecord,
    pub project: SavedProjectRecord,
    pub measurements: Vec<SpecimenMeasurement>,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "view"))]
pub enum Specimen {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedSpecimenRecord,
        links: SimpleLinks,
    },
    // Rather than just wrapping the `SpecimenRecordDetailed`, we destructure its fields so that
    // we have a `Project` rather than a `ProjectRecord`
    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedSpecimenRecord,
        links: SimpleLinks,
        project: Project,
        measurements: Vec<SpecimenMeasurement>,
    },
}

impl SimpleLinks {
    fn for_specimen(id: Id) -> Self {
        Self::from_str_and_id("/specimens", id)
    }
}

impl Specimen {
    pub fn record(&self) -> &SavedSpecimenRecord {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed { record, .. } => record,
        }
    }

    pub fn from_record(record: SavedSpecimenRecord) -> Self {
        Self::Compact {
            links: SimpleLinks::for_specimen(record.id),
            record,
        }
    }

    pub fn from_detailed_record(
        SavedSpecimenRecordDetailed {
            specimen,
            project,
            measurements,
        }: SavedSpecimenRecordDetailed,
    ) -> Self {
        Self::Detailed {
            links: SimpleLinks::for_specimen(specimen.id),
            record: specimen,
            project: Project::from_record(project),
            measurements,
        }
    }
}
