use macro_attributes::base_model;
use nonempty::NonemptyVec;
pub use query::{CdnaField, CdnaPredicate, CdnaPredicateInner, CdnaQuery, SimpleCdnaQuery};
use uuid::Uuid;

use crate::{
    cdna::{measurement::CdnaMeasurement, record::CdnaRecord},
    id::{Id, NoId},
    nucleic_acid_measurement::NewNucleicAcidMeasurement,
    simple_links::SimpleLinks,
    suspension_pool::TaggedSpecimen,
};

pub mod measurement;
mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use nonempty::NonemptyString;
    use positive::PositiveI32;
    use uuid::Uuid;

    use crate::tenx_assay::LibraryType;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "cdna"))]
    pub struct CdnaRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub library_type: LibraryType,
        pub prepared_at: Timestamp,
        pub gem_well_id: Option<Uuid>,
        pub n_amplification_cycles: PositiveI32,
        pub additional_data: Option<serde_json::Value>,
    }
}

pub type NewCdnaRecord = CdnaRecord<NoId>;

pub type SavedCdnaRecord = CdnaRecord<Id>;

#[base_model]
pub struct NewCdna {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewCdnaRecord,
    pub measurements: Vec<NewNucleicAcidMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
}

#[base_model]
pub struct CdnaUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewCdnaRecord,
    #[cfg_attr(feature = "serde", serde(default))]
    pub measurements: Vec<NewNucleicAcidMeasurement>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub preparers: Vec<Uuid>,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "view"))]
pub enum Cdna {
    Compact {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedCdnaRecord,
        links: SimpleLinks,
    },

    Detailed {
        #[cfg_attr(feature = "serde", serde(flatten))]
        record: SavedCdnaRecord,
        links: SimpleLinks,
        specimens: Vec<TaggedSpecimen>,
        measurements: Vec<CdnaMeasurement>,
        preparers: Vec<Uuid>,
    },
}

impl SimpleLinks {
    pub fn for_cdna(id: Id) -> Self {
        Self::from_str_and_id("/cdna", id)
    }
}

impl Cdna {
    #[must_use]
    pub fn from_record(record: SavedCdnaRecord) -> Self {
        Self::Compact {
            links: SimpleLinks::for_cdna(record.id),
            record,
        }
    }

    #[must_use]
    pub fn record(&self) -> &SavedCdnaRecord {
        match self {
            Self::Compact { record, .. } => record,
            Self::Detailed { record, .. } => record,
        }
    }
}
