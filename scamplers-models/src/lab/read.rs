#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{labs, people};
use uuid::Uuid;

use crate::{lab::common::Fields, links::Links, person::PersonSummary};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct LabSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: Fields,
    #[cfg_attr(feature = "typescript", ts(inline))]
    links: Links,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = labs, base_query = labs::table.inner_join(people::table)))]
pub struct Lab {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: LabSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pi: PersonSummary,
}
