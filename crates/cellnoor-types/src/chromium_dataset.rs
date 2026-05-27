use macro_attributes::{base_model, unit_enum};
use nonempty::NonemptyVec;
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

#[unit_enum]
pub enum ChromiumDatasetCmdline {
    #[serde(rename = "cellranger-arc count")]
    #[strum(serialize = "cellranger-arc count")]
    CellrangerarcCount,
    #[serde(rename = "cellranger-atac count")]
    #[strum(serialize = "cellranger-atac count")]
    CellrangeratacCount,
    #[serde(rename = "cellranger count")]
    #[strum(serialize = "cellranger count")]
    CellrangerCount,
    #[serde(rename = "cellranger multi")]
    #[strum(serialize = "cellranger multi")]
    CellrangerMulti,
    #[serde(rename = "cellranger vdj")]
    #[strum(serialize = "cellranger vdj")]
    CellrangerVdj,
}

pub type NewChromiumDatasetRecord = ChromiumDatasetRecord<NoId>;

pub type SavedChromiumDatasetRecord = ChromiumDatasetRecord<Id>;

pub type ChromiumDatasetUpdate = NewChromiumDatasetRecord;

#[base_model]
pub struct NewChromiumDataset {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: NewChromiumDatasetRecord,
    pub library_ids: NonemptyVec<Uuid>,
    pub cmdline: ChromiumDatasetCmdline,
}

#[base_model]
pub struct ChromiumDatasetCompact {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumDatasetRecord,
    pub links: SimpleLinks,
}

#[base_model]
pub struct ChromiumDatasetDetailed {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub record: SavedChromiumDatasetRecord,
    pub links: SimpleLinks,
    pub libraries: Vec<LibraryCompact>,
    pub specimens: Vec<TaggedSpecimen>,
    pub raw_file_paths: Vec<String>,
}
