#[cfg(feature = "app")]
use cellnoor_schema::{labs, people};
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    lab::common::LabFields,
    links::Links,
    person::{self},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct LabSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: LabFields,
    links: Links,
}
impl LabSummary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name.as_ref()
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = labs, base_query = labs::table.inner_join(people::table)))]
pub struct Lab {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: LabSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pi: person::PersonSummary,
}
impl Lab {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.summary.id()
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.summary.name()
    }
}
