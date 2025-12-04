#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{chromium_runs, gem_pools, tenx_assays};
use uuid::Uuid;

use crate::{
    chromium_run::common::{ChromiumRunFields, GemPoolFields},
    tenx_assay::TenxAssay,
};

#[select]
#[cfg_attr(feature = "app", derive(Identifiable))]
#[cfg_attr(feature = "app", diesel(table_name = gem_pools))]
pub struct GemPoolSummary {
    id: Uuid,
    chromium_run_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: GemPoolFields,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = chromium_runs))]
pub struct ChromiumRunSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: ChromiumRunFields,
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = chromium_runs::table.inner_join(tenx_assays::table)))]
pub struct ChromiumRun {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: ChromiumRunSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    assay: TenxAssay,
}
