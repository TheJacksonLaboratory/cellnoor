#[cfg(feature = "app")]
use cellnoor_schema::chromium_datasets;
#[cfg(feature = "app")]
use diesel::pg::Pg;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    chromium_dataset::common::ChromiumDatasetFields, project::Project, tenx_assay::TenxAssay,
};

// Manually derive everything because the query is too complicated to write here
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[cfg_attr(
    feature = "app",
    derive(diesel::Selectable, diesel::Queryable, schemars::JsonSchema)
)]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets, check_for_backend(Pg)))]
pub struct ChromiumDataset {
    pub id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    pub delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub assay: TenxAssay,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub project: Project,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: ChromiumDatasetLinks,
}

impl ChromiumDataset {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn delivered_at(&self) -> Timestamp {
        self.delivered_at
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name.as_ref()
    }

    #[must_use]
    pub fn project_id(&self) -> Uuid {
        self.project.id()
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetLinks {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "specimens")]
    pub specimens_link: String,
    #[serde(rename = "libraries")]
    pub libraries_link: String,
    #[serde(rename = "raw_files")]
    pub raw_file_links: Vec<Option<String>>,
    #[serde(rename = "parsed_files")]
    pub parsed_file_links: Vec<Option<String>>,
}
