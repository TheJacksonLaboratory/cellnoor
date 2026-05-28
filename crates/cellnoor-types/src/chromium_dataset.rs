use macro_attributes::{base_model, select};
use nonempty::{NonemptyString, NonemptyVec};
pub use query::{
    ChromiumDatasetField, ChromiumDatasetPredicate, ChromiumDatasetPredicateInner,
    ChromiumDatasetQuery, SimpleChromiumDatasetQuery,
};
use uuid::Uuid;

use crate::{
    chromium_dataset::record::ChromiumDatasetRecord,
    id::{Id, NoId},
    library::LibraryCompact,
    simple_links::SimpleLinks,
    suspension_pool::TaggedSpecimen,
};

mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;
    use nonempty::NonemptyString;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "chromium_dataset"))]
    pub struct ChromiumDatasetRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub name: NonemptyString,
        pub delivered_at: Timestamp,
    }
}

pub type NewChromiumDatasetRecord = ChromiumDatasetRecord<NoId>;

pub type SavedChromiumDatasetRecord = ChromiumDatasetRecord<Id>;

pub type ChromiumDatasetUpdate = NewChromiumDatasetRecord;

#[base_model]
pub struct NewChromiumDataset {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewChromiumDatasetRecord,
    pub library_ids: NonemptyVec<Uuid>,
}

#[base_model]
pub struct ChromiumDatasetCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumDatasetRecord,
    pub links: SimpleLinks,
}

#[base_model]
pub struct ChromiumDatasetDetailedLinks {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub simple: SimpleLinks,
    pub raw_files: Vec<String>,
}

#[select]
#[cfg_attr(
    feature = "postgres-types",
    postgres(name = "chromium_dataset_parsed_file")
)]
pub struct ChromiumDatasetParsedFile {
    pub dataset_id: Uuid,
    pub path: NonemptyString,
    pub data: serde_json::Value,
}

#[base_model]
pub struct ChromiumDatasetDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumDatasetRecord,
    pub links: ChromiumDatasetDetailedLinks,
    pub libraries: Vec<LibraryCompact>,
    pub specimens: Vec<TaggedSpecimen>,
    pub data: Vec<ChromiumDatasetParsedFile>,
}
