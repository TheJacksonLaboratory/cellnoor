#[cfg(feature = "app")]
use cellnoor_schema::chromium_datasets;
#[cfg(feature = "app")]
use diesel::pg::Pg;
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    chromium_dataset::common::ChromiumDatasetFields, lab::LabSummary, links::Links,
    tenx_assay::TenxAssay,
};

// Manually derive everything because the query is too complicated to write here
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "app", derive(diesel::Selectable, diesel::Queryable))]
#[cfg_attr(feature = "typescript", derive(::ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(optional_fields))]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets, check_for_backend(Pg)))]
pub struct ChromiumDatasetSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    links: Links,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(embed))]
    assay: TenxAssay,
}

impl ChromiumDatasetSummary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }
}

// Manually derive everything because the query is too complicated to write here
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(feature = "app", derive(diesel::Selectable, diesel::Queryable))]
#[cfg_attr(feature = "typescript", derive(::ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(optional_fields))]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets, check_for_backend(Pg)))]
pub struct ChromiumDataset {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: ChromiumDatasetSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    lab: LabSummary,
}

impl ChromiumDataset {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.summary.id()
    }
}
