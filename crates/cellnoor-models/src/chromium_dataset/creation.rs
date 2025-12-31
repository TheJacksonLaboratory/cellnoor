#[cfg(feature = "app")]
use cellnoor_schema::chromium_datasets;
use jiff::Timestamp;
use macro_attributes::{insert, simple_enum};
use macros::{impl_enum_from_sql, impl_enum_to_sql};
use uuid::Uuid;

use crate::chromium_dataset::common::ChromiumDatasetFields;
#[cfg(feature = "app")]
use crate::utils::{EnumFromSql, EnumToSql};

pub mod metrics;

#[simple_enum]
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

#[cfg(feature = "app")]
impl EnumFromSql for ChromiumDatasetCmdline {}
impl_enum_from_sql!(ChromiumDatasetCmdline);

#[cfg(feature = "app")]
impl EnumToSql for ChromiumDatasetCmdline {}
impl_enum_to_sql!(ChromiumDatasetCmdline);

#[insert]
#[cfg_attr(feature = "app", diesel(table_name = chromium_datasets))]
pub struct ChromiumDatasetCreation {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    delivered_at: Timestamp,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    library_ids: Vec<Uuid>,
    #[cfg_attr(feature = "app", diesel(skip_insertion))]
    cmdline: ChromiumDatasetCmdline,
}

impl ChromiumDatasetCreation {
    #[must_use]
    pub fn cmdline(&self) -> ChromiumDatasetCmdline {
        self.cmdline
    }

    #[must_use]
    pub fn library_ids(&self) -> &[Uuid] {
        &self.library_ids
    }

    #[must_use]
    pub fn delivered_at(&self) -> Timestamp {
        self.delivered_at
    }
}
