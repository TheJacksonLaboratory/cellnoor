#[cfg(feature = "app")]
use cellnoor_schema::{specimens, suspensions};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    links::Links,
    specimen::SpecimenSummary,
    suspension::common::{SuspensionContent, SuspensionFields},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = suspensions))]
pub struct SuspensionSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: SuspensionFields,
    #[cfg_attr(feature = "app", diesel(deserialize_as = jiff_diesel::NullableTimestamp))]
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    created_at: Option<Timestamp>,
    target_cell_recovery: i64,
    lysis_duration_minutes: Option<f32>,
    content: SuspensionContent,
    links: Links,
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
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = suspensions::table.inner_join(specimens::table)))]
pub struct Suspension {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: SuspensionSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    parent_specimen: SpecimenSummary,
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
}
