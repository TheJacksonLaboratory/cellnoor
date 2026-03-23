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
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(embed))]
    assay: TenxAssay,
    #[cfg_attr(feature = "app", diesel(embed))]
    project: Project,
    #[cfg_attr(feature = "app", diesel(embed))]
    links: ChromiumDatasetLinks,
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
    self_link: String,
    #[serde(rename = "specimens")]
    specimens_link: String,
    #[serde(rename = "libraries")]
    libraries_link: String,
    #[serde(rename = "files")]
    file_links: Vec<Option<String>>,
}
