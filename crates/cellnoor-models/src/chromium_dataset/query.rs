#![allow(clippy::implicit_clone)]

#[cfg(feature = "app")]
use cellnoor_schema::chromium_datasets;
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::{specimen::SpecimenFilter, tenx_assay::TenxAssayFilter};

#[filter]
pub struct ChromiumDatasetFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
    pub specimen: Option<SpecimenFilter>,
    pub assay: Option<TenxAssayFilter>,
    pub lab_ids: Option<Vec<Uuid>>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub delivered_before: Option<Timestamp>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub delivered_after: Option<Timestamp>,
}

#[order_by(chromium_datasets)]
#[allow(non_camel_case_types)]
pub enum ChromiumDatasetOrderBy {
    id { descending: Option<bool> },
    name { descending: Option<bool> },
    lab_id { descending: Option<bool> },
    delivered_at { descending: Option<bool> },
}

impl Default for ChromiumDatasetOrderBy {
    fn default() -> Self {
        Self::delivered_at {
            descending: Some(true),
        }
    }
}

#[cfg(feature = "app")]
pub type ChromiumDatasetQuery = generic_query::Query<ChromiumDatasetFilter, ChromiumDatasetOrderBy>;

uuid_newtype!(ChromiumDatasetId, "/{id}");

uuid_newtype!(ChromiumDatasetIdSpecimens, "/{id}/specimens");

uuid_newtype!(ChromiumDatasetIdLibraries, "/{id}/libraries");

uuid_newtype!(ChromiumDatasetIdWebSummaries, "/{id}/web-summaries");

uuid_newtype!(ChromiumDatasetIdMetrics, "/{id}/metrics-files");

#[derive(Debug, Clone, ::serde::Deserialize, ::serde::Serialize)]
#[cfg_attr(feature = "app", derive(axum_extra::routing::TypedPath))]
#[cfg_attr(
    feature = "app",
    typed_path("/{dataset_id}/web-summaries/{directory}/{filename}")
)]
pub struct ChromiumDatasetWebSummaryFilename(pub ChromiumDatasetId, pub String, pub String);

#[derive(Debug, Clone, ::serde::Deserialize, ::serde::Serialize)]
#[cfg_attr(feature = "app", derive(axum_extra::routing::TypedPath))]
#[cfg_attr(
    feature = "app",
    typed_path("/{dataset_id}/metrics-files/{directory}/{filename}")
)]
pub struct ChromiumDatasetMetricsFilename(pub ChromiumDatasetId, pub String, pub String);
