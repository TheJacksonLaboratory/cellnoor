#[cfg(feature = "app")]
use cellnoor_schema::{chromium_runs, gem_pools, tenx_assays};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    chromium_run::common::{ChromiumRunFields, GemPoolFields},
    tenx_assay::TenxAssay,
};

#[select]
#[cfg_attr(feature = "app", derive(Identifiable))]
#[cfg_attr(feature = "app", diesel(table_name = gem_pools, base_query = gem_pools::table.inner_join(chromium_runs::table)))]
pub struct GemPoolSummary {
    pub id: Uuid,
    pub chromium_run_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: GemPoolFields,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: GemPoolLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = gem_pools))]
pub struct GemPoolLinks {
    #[serde(rename = "self")]
    pub self_link: String,
}

impl GemPoolSummary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn chromium_run_id(&self) -> Uuid {
        self.chromium_run_id
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_runs))]
pub struct ChromiumRunSummary {
    pub id: Uuid,
    pub project_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: ChromiumRunFields,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: ChromiumRunLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_runs))]
pub struct ChromiumRunLinks {
    #[serde(rename = "self")]
    pub self_link: String,
}

impl ChromiumRunSummary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn project_id(&self) -> Uuid {
        self.project_id
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = gem_pools::table.inner_join(chromium_runs::table)))]
pub struct GemPool {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub summary: GemPoolSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub chromium_run: ChromiumRunSummary,
}

impl GemPool {
    #[must_use]
    pub fn chromium_run_at(&self) -> Timestamp {
        self.chromium_run.inner.run_at
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = chromium_runs::table.inner_join(tenx_assays::table)))]
pub struct ChromiumRun {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub summary: ChromiumRunSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub assay: TenxAssay,
}
