use jiff::Timestamp;
use macro_attributes::base_model;
#[cfg(feature = "app")]
use scamplers_schema::chromium_datasets;
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

use crate::chromium_dataset::{
    common::{ChromiumDatasetFields, ParsedMetrics},
    creation::metrics::{json::Json, multi_row_csv::MultiRowCsv, single_row_csv::SingleRowCsv},
};

pub(super) mod metrics;

#[base_model]
#[derive(serde::Deserialize)]
#[serde(tag = "cmdline")]
#[derive(strum::IntoStaticStr)]
pub enum ChromiumDatasetCreation {
    #[serde(rename = "cellranger-arc count")]
    #[strum(serialize = "cellranger-arc count")]
    CellrangerarcCount {
        #[serde(flatten)]
        inner: ChromiumDatasetFields,
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        delivered_at: Timestamp,
        library_ids: Vec<Uuid>,
        #[serde(deserialize_with = "deserialize_single_row_csv")]
        metrics_file: ParsedMetrics,
    },
    #[serde(rename = "cellranger-atac count")]
    #[strum(serialize = "cellranger-atac count")]
    CellrangeratacCount {
        #[serde(flatten)]
        inner: ChromiumDatasetFields,
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        delivered_at: Timestamp,
        library_ids: Vec<Uuid>,
        #[serde(deserialize_with = "deserialize_json")]
        metrics_file: ParsedMetrics,
    },
    #[serde(rename = "cellranger count")]
    #[strum(serialize = "cellranger count")]
    CellrangerCount {
        #[serde(flatten)]
        inner: ChromiumDatasetFields,
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        delivered_at: Timestamp,
        library_ids: Vec<Uuid>,
        #[serde(deserialize_with = "deserialize_single_row_csv")]
        metrics_file: ParsedMetrics,
    },
    #[serde(rename = "cellranger multi")]
    #[strum(serialize = "cellranger multi")]
    CellrangerMulti {
        #[serde(flatten)]
        inner: ChromiumDatasetFields,
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        delivered_at: Timestamp,
        library_ids: Vec<Uuid>,
        #[serde(deserialize_with = "deserialize_multi_row_csv")]
        metrics_files: ParsedMetrics,
    },
    #[serde(rename = "cellranger vdj")]
    #[strum(serialize = "cellranger vdj")]
    CellrangerVdj {
        #[serde(flatten)]
        inner: ChromiumDatasetFields,
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        delivered_at: Timestamp,
        library_ids: Vec<Uuid>,
        #[serde(deserialize_with = "deserialize_single_row_csv")]
        metrics_file: ParsedMetrics,
    },
}

#[cfg(feature = "app")]
#[derive(diesel::Insertable)]
#[diesel(table_name = chromium_datasets)]
pub struct GenericChromiumDatasetCreation {
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumDatasetFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp))]
    delivered_at: Timestamp,
    parsed_metrics_files: ParsedMetrics,
}

impl ChromiumDatasetCreation {
    pub fn cmdline(&self) -> &str {
        self.into()
    }

    pub fn library_ids(&self) -> &[Uuid] {
        match self {
            Self::CellrangerarcCount { library_ids, .. }
            | Self::CellrangeratacCount { library_ids, .. }
            | Self::CellrangerCount { library_ids, .. }
            | Self::CellrangerMulti { library_ids, .. }
            | Self::CellrangerVdj { library_ids, .. } => library_ids,
        }
    }

    pub fn delivered_at(&self) -> Timestamp {
        match self {
            Self::CellrangerarcCount { delivered_at, .. }
            | Self::CellrangeratacCount { delivered_at, .. }
            | Self::CellrangerCount { delivered_at, .. }
            | Self::CellrangerMulti { delivered_at, .. }
            | Self::CellrangerVdj { delivered_at, .. } => *delivered_at,
        }
    }

    pub fn data_path(&self) -> &str {
        match self {
            Self::CellrangerarcCount { inner, .. }
            | Self::CellrangeratacCount { inner, .. }
            | Self::CellrangerCount { inner, .. }
            | Self::CellrangerMulti { inner, .. }
            | Self::CellrangerVdj { inner, .. } => inner.data_path.as_ref(),
        }
    }

    pub fn split_for_insertion(self) -> (GenericChromiumDatasetCreation, Vec<Uuid>) {
        match self {
            Self::CellrangerarcCount {
                inner,
                delivered_at,
                library_ids,
                metrics_file: parsed_metrics_files,
            }
            | Self::CellrangeratacCount {
                inner,
                delivered_at,
                library_ids,
                metrics_file: parsed_metrics_files,
            }
            | Self::CellrangerCount {
                inner,
                delivered_at,
                library_ids,
                metrics_file: parsed_metrics_files,
            }
            | Self::CellrangerMulti {
                inner,
                delivered_at,
                library_ids,
                metrics_files: parsed_metrics_files,
            }
            | Self::CellrangerVdj {
                inner,
                delivered_at,
                library_ids,
                metrics_file: parsed_metrics_files,
            } => (
                GenericChromiumDatasetCreation {
                    inner,
                    delivered_at,
                    parsed_metrics_files,
                },
                library_ids,
            ),
        }
    }
}

fn deserialize_single_row_csv<'de, D>(deserializer: D) -> Result<ParsedMetrics, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(SingleRowCsv::deserialize(deserializer)?.into())
}

fn deserialize_multi_row_csv<'de, D>(deserializer: D) -> Result<ParsedMetrics, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Vec::<MultiRowCsv>::deserialize(deserializer)?.into())
}

fn deserialize_json<'de, D>(deserializer: D) -> Result<ParsedMetrics, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Json::deserialize(deserializer)?.into())
}
