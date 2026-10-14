use std::collections::HashMap;

use csvranger::TenxCsvValue;
use macro_attributes::{base_model, select};
#[cfg(all(feature = "postgres-types", feature = "schemars"))]
use postgres_types::Json;
pub use query::{
    ChromiumDatasetField, ChromiumDatasetPredicate, ChromiumDatasetPredicateInner,
    ChromiumDatasetQuery, SimpleChromiumDatasetQuery,
};
use uuid::Uuid;

use crate::{
    Relation,
    chromium_dataset::record::ChromiumDatasetRecord,
    file_tree::FileTreeNode,
    id::{Id, NoId},
    library::LibraryCompact,
    nonempty::{NonemptyString, NonemptyVec},
    simple_links::SimpleLinks,
    suspension_pool::TaggedSpecimen,
    tenx_assay::TenxAssay,
};

mod query;

mod record {
    use jiff::Timestamp;
    use macro_attributes::select;

    use crate::nonempty::NonemptyString;

    #[select]
    #[cfg_attr(feature = "postgres-types", postgres(name = "chromium_dataset"))]
    pub struct ChromiumDatasetRecord<T> {
        #[cfg_attr(feature = "serde", serde(flatten))]
        pub id: T,
        pub name: NonemptyString,
        pub delivered_at: Timestamp,
    }
}

impl<T> Relation for ChromiumDatasetRecord<T> {
    const NAME: &'static str = "chromium_dataset";
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
    pub file_dir: String,
    pub raw_files: Vec<String>,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "format", rename_all = "snake_case"))]
pub enum ChromiumDatasetParsedFileData {
    SingleRowCsv {
        data: HashMap<String, TenxCsvValue>,
    },
    MultiRowCsv {
        data: Vec<HashMap<String, TenxCsvValue>>,
    },
    Json {
        data: HashMap<String, serde_json::Value>,
    },
}

#[select]
#[cfg_attr(
    feature = "postgres-types",
    postgres(name = "chromium_dataset_parsed_file")
)]
pub struct ChromiumDatasetParsedFileRecord {
    pub dataset_id: Uuid,
    pub path: NonemptyString,
    #[cfg(feature = "postgres-types")]
    #[cfg_attr(feature = "schemars", schemars(with = "ChromiumDatasetParsedFileData"))]
    pub data: Json<ChromiumDatasetParsedFileData>,
    #[cfg(not(feature = "postgres-types"))]
    pub data: ChromiumDatasetParsedFileData,
}

#[base_model]
pub struct ChromiumDatasetDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumDatasetRecord,
    pub assay: TenxAssay,
    pub links: ChromiumDatasetDetailedLinks,
    pub libraries: Vec<LibraryCompact>,
    pub specimens: Vec<TaggedSpecimen>,
    pub files: FileTreeNode<ChromiumDatasetParsedFileData>,
}
