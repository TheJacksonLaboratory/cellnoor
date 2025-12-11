#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{chromium_datasets, labs};
use uuid::Uuid;

use crate::{
    chromium_dataset::common::{ChromiumDatasetFields, ParsedMetrics},
    lab::LabSummary,
    links::Links,
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    links: Links,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    parsed_metrics_files: ParsedMetrics,
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = chromium_datasets::table.inner_join(labs::table)))]
pub struct ChromiumDataset {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: ChromiumDatasetSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    lab: LabSummary,
}
