use jiff::Timestamp;
use macro_attributes::{insert, json};
use non_empty::NonEmptyVec;
#[cfg(feature = "app")]
use scamplers_schema::chromium_datasets;
use uuid::Uuid;

use crate::chromium_dataset::{
    common::ChromiumDatasetFields,
    creation::metrics::{Json, MultiRowCsv, SingleRowCsv},
};

mod metrics;

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct CellrangerarcCountDatasetCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    library_ids: Vec<Uuid>,
    metrics: SingleRowCsv,
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct CellrangeratacCountDatasetCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    library_ids: Vec<Uuid>,
    metrics: Json,
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct CellrangerCountDatasetCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    library_ids: Vec<Uuid>,
    metrics: SingleRowCsv,
}

#[json]
pub struct MultiRowCsvGroup(NonEmptyVec<MultiRowCsv, { usize::MAX }>);

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct CellrangerMultiDatasetCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    library_ids: Vec<Uuid>,
    metrics: MultiRowCsvGroup,
}

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct CellrangerVdjDatasetCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    library_ids: Vec<Uuid>,
    metrics: SingleRowCsv,
}
