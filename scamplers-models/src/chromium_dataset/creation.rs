use jiff::Timestamp;
use macro_attributes::{base_model, insert};
#[cfg(feature = "app")]
use scamplers_schema::chromium_datasets;
use uuid::Uuid;

use crate::chromium_dataset::{
    common::ChromiumDatasetFields,
    creation::metrics::{json::Json, multi_row_csv::MultiRowCsv, single_row_csv::SingleRowCsv},
};

pub(super) mod metrics;

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
    #[cfg_attr(feature = "app", diesel(serialize_as = super::common::ParsedMetrics))]
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
    #[cfg_attr(feature = "app", diesel(serialize_as = super::common::ParsedMetrics))]
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
    #[cfg_attr(feature = "app", diesel(serialize_as = super::common::ParsedMetrics))]
    metrics: SingleRowCsv,
}

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
    #[cfg_attr(feature = "app", diesel(serialize_as = super::common::ParsedMetrics))]
    metrics: Vec<MultiRowCsv>,
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
    #[cfg_attr(feature = "app", diesel(serialize_as = super::common::ParsedMetrics))]
    metrics: SingleRowCsv,
}

#[base_model]
#[derive(serde::Deserialize)]
#[serde(tag = "cmdline")]
#[derive(strum::IntoStaticStr)]
pub enum ChromiumDatasetCreation {
    #[serde(rename = "cellranger-arc count")]
    #[strum(serialize = "cellranger-arc count")]
    CellrangerarcCount(CellrangerarcCountDatasetCreation),
    #[serde(rename = "cellranger-atac count")]
    #[strum(serialize = "cellranger-atac count")]
    CellrangeratacCount(CellrangeratacCountDatasetCreation),
    #[serde(rename = "cellranger count")]
    #[strum(serialize = "cellranger count")]
    CellrangerCount(CellrangerCountDatasetCreation),
    #[serde(rename = "cellranger multi")]
    #[strum(serialize = "cellranger multi")]
    CellrangerMulti(CellrangerMultiDatasetCreation),
    #[serde(rename = "cellranger vdj")]
    #[strum(serialize = "cellranger vdj")]
    CellrangerVdj(CellrangerVdjDatasetCreation),
}

impl ChromiumDatasetCreation {
    pub fn cmdline(&self) -> &str {
        self.into()
    }
}
