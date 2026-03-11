#[cfg(feature = "app")]
use cellnoor_schema::{cdna, gem_pools};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{chromium_run::GemPoolSummary, nucleic_acid::cdna::common::CdnaFields};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = cdna))]
pub struct CdnaSummary {
    id: Uuid,
    project_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: CdnaFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::Timestamp))]
    prepared_at: Timestamp,
    n_amplification_cycles: i32,
    #[cfg_attr(feature = "app", diesel(embed))]
    links: CdnaLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = cdna))]
pub struct CdnaLinks {
    #[serde(rename = "self")]
    self_link: String,
    #[serde(rename = "measurements")]
    measurements_link: String,
    #[serde(rename = "libraries")]
    libraries_link: String,
}

impl CdnaSummary {
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
#[cfg_attr(feature = "app", diesel(table_name = cdna, base_query = cdna::table.inner_join(gem_pools::table)))]
pub struct Cdna {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: CdnaSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    gem_pool: GemPoolSummary,
}

impl Cdna {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.summary.id()
    }

    #[must_use]
    pub fn prepared_at(&self) -> Timestamp {
        self.summary.prepared_at
    }
}
