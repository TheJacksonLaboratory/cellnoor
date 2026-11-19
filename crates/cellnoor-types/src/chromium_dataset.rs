use std::collections::HashMap;

use csvranger::TenxCsvValue;
use indexmap::IndexMap;
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
    cdna::{CdnaCompact, creation::LibraryType},
    chromium_dataset::record::ChromiumDatasetRecord,
    chromium_run::{ChromiumRunCompact, creation::ocm::OcmBarcodeId},
    file_tree::FileTreeNode,
    id::{Id, NoId},
    library::LibraryCompact,
    multiplexing_tag::MultiplexingTag,
    nonempty::{NonemptyString, NonemptyVec},
    reference_package::ReferencePackage,
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
        pub reference_package_name: NonemptyString,
        pub pipeline_name: NonemptyString,
        pub pipeline_command: NonemptyString,
        pub pipeline_version: NonemptyString,
        pub additional_data: Option<serde_json::Value>,
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
pub struct TransposedCsvRow {
    #[cfg_attr(
        feature = "serde",
        serde(rename = "Category", alias = "Library or Sample")
    )]
    pub category: String,
    #[cfg_attr(feature = "serde", serde(rename = "Library Type"))]
    pub library_type: LibraryType,
    #[cfg_attr(feature = "serde", serde(rename = "Grouped By"))]
    pub grouped_by: String,
    #[cfg_attr(feature = "serde", serde(rename = "Group Name"))]
    pub group_name: String,
    #[cfg_attr(feature = "serde", serde(rename = "Metric Name"))]
    pub metric_name: String,
    #[cfg_attr(feature = "serde", serde(rename = "Metric Value"))]
    pub metric_value: TenxCsvValue,
}

#[base_model]
#[cfg_attr(feature = "serde", serde(tag = "format", rename_all = "snake_case"))]
pub enum ChromiumDatasetParsedFileData {
    StandardCsv {
        // We have to store the column order because postgres JSONB columns
        // don't preserve key order
        columns: Vec<String>,
        data: Vec<HashMap<String, TenxCsvValue>>,
    },
    TransposedCsv {
        data: Vec<TransposedCsvRow>,
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

// We can't use base_model for these because you can't deserialize a &'static
// str
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct KeySpecimenMetrics {
    pub specimen_id: Uuid,
    pub specimen_readable_id: NonemptyString,
    pub specimen_name: NonemptyString,
    // IndexMap so that order is maintained when serializing
    pub renamed_metrics: IndexMap<&'static str, TenxCsvValue>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ChromiumDatasetDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumDatasetRecord,
    pub reference_package: ReferencePackage,
    pub assay: TenxAssay,
    pub chromium_run: ChromiumRunCompact,
    pub links: ChromiumDatasetDetailedLinks,
    pub libraries: Vec<LibraryCompact>,
    pub cdna: Vec<CdnaCompact>,
    pub specimens: Vec<TaggedSpecimen>,
    // IndexMap so that order is maintained when serializing
    pub key_metrics: IndexMap<LibraryType, Vec<KeySpecimenMetrics>>,
    pub files: Vec<FileTreeNode<ChromiumDatasetParsedFileData>>,
}
