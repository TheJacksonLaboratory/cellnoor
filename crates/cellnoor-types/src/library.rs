use macro_attributes::base_model;
use nonempty::NonemptyVec;
pub use query::{
    LibraryField, LibraryPredicate, LibraryPredicateInner, LibraryQuery, SimpleLibraryQuery,
};
use uuid::Uuid;

use crate::{
    id::{Id, NoId},
    library::{measurement::LibraryMeasurement, record::LibraryRecord},
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

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "library"))]
    pub struct LibraryRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub readable_id: NonemptyString,
        pub cdna_id: Uuid,
        pub single_index_set_name: Option<String>,
        pub dual_index_set_name: Option<String>,
        pub number_of_sample_index_pcr_cycles: PositiveI32,
        pub target_reads_per_cell: Option<PositiveI32>,
        pub prepared_at: Timestamp,
        pub additional_data: Option<serde_json::Value>,
    }
}

pub type NewLibraryRecord = LibraryRecord<NoId>;

pub type SavedLibraryRecord = LibraryRecord<Id>;

#[base_model]
pub struct NewLibrary {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewLibraryRecord,
    pub measurements: Vec<NewNucleicAcidMeasurement>,
    pub preparers: NonemptyVec<Uuid>,
}

#[base_model]
pub struct LibraryUpdate {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewLibraryRecord,
    pub measurements: Option<Vec<NewNucleicAcidMeasurement>>,
    pub preparers: Option<Vec<Uuid>>,
}

#[base_model]
pub struct LibraryCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedLibraryRecord,
    pub links: SimpleLinks,
}

#[base_model]
pub struct LibraryDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedLibraryRecord,
    pub links: SimpleLinks,
    pub specimens: Vec<TaggedSpecimen>,
    pub measurements: Vec<LibraryMeasurement>,
    pub preparers: Vec<Uuid>,
}
