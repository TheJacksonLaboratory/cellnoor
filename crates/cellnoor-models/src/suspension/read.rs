#[cfg(feature = "app")]
use cellnoor_schema::{specimens, suspensions};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    specimen::SpecimenSummary,
    suspension::common::{SuspensionContent, SuspensionFields},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = suspensions))]
pub struct SuspensionSummary {
    pub id: Uuid,
    pub project_id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: SuspensionFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::NullableTimestamp))]
    pub created_at: Option<Timestamp>,
    pub target_cell_recovery: Option<i64>,
    pub lysis_duration_minutes: Option<f32>,
    pub content: SuspensionContent,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: SuspensionLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = suspensions))]
pub struct SuspensionLinks {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "measurements")]
    pub measurements_link: String,
}

impl SuspensionSummary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn created_at(&self) -> Option<Timestamp> {
        self.created_at
    }

    #[must_use]
    pub fn project_id(&self) -> Uuid {
        self.project_id
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = suspensions::table.inner_join(specimens::table)))]
pub struct Suspension {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub summary: SuspensionSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub parent_specimen: SpecimenSummary,
}

impl Suspension {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.summary.id()
    }

    #[must_use]
    pub fn created_at(&self) -> Option<Timestamp> {
        self.summary.created_at()
    }

    #[must_use]
    pub fn parent_specimen_received_at(&self) -> Timestamp {
        self.parent_specimen.received_at()
    }

    #[must_use]
    pub fn content(&self) -> SuspensionContent {
        self.summary.content
    }
}
