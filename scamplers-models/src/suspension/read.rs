#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{specimens, suspensions};
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
    content: SuspensionContent,
    links: Links,
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
