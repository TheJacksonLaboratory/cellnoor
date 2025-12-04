#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{cdna, gem_pools};
use uuid::Uuid;

use crate::{chromium_run::GemPoolSummary, nucleic_acid::cdna::common::CdnaFields};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = cdna))]
pub struct CdnaSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: CdnaFields,
    #[cfg_attr(feature = "app", diesel(serialize_as = jiff_diesel::Timestamp, deserialize_as = jiff_diesel::Timestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "String"))]
    prepared_at: Timestamp,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = cdna, base_query = cdna::table.inner_join(gem_pools::table)))]
pub struct Cdna {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: CdnaSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    gem_pool: GemPoolSummary,
}
