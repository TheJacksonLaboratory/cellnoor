use jiff::Timestamp;
use macro_attributes::{base_model, select};
use nonempty::NonemptyString;
use positive::PositiveI32;
pub use query::{CdnaField, CdnaPredicate, CdnaPredicateInner, CdnaQuery, SimpleCdnaQuery};
use uuid::Uuid;

use crate::{
    cdna::{
        creation::{CdnaSimpleFields, LibraryType},
        measurement::CdnaMeasurement,
    },
    nucleic_acid_measurement::NewNucleicAcidMeasurement,
    simple_links::SimpleLinks,
    suspension_pool::TaggedSpecimen,
};

pub mod creation;
pub mod measurement;
mod query;

#[base_model]
pub struct CdnaUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: CdnaSimpleFields,
    pub measurements: Option<Vec<NewNucleicAcidMeasurement>>,
    pub preparers: Option<Vec<Uuid>>,
}

#[select]
#[cfg_attr(feature = "postgres-types", postgres(name = "cdna"))]
pub struct SavedCdnaRecord {
    pub id: Uuid,
    pub readable_id: NonemptyString,
    pub library_type: LibraryType,
    pub prepared_at: Timestamp,
    pub gem_well_id: Option<Uuid>,
    pub n_amplification_cycles: Option<PositiveI32>,
    pub additional_data: Option<serde_json::Value>,
}

#[base_model]
pub struct CdnaCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedCdnaRecord,
    pub links: SimpleLinks,
}

#[base_model]
pub struct CdnaDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedCdnaRecord,
    pub links: SimpleLinks,
    pub specimens: Vec<TaggedSpecimen>,
    pub measurements: Vec<CdnaMeasurement>,
    pub preparers: Vec<Uuid>,
}
