#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{labs, people};
use uuid::Uuid;

use crate::{
    lab::common::Fields,
    links::Links,
    person::{self},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = labs))]
pub struct Summary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: Fields,

    links: Links,
}
impl Summary {
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
    summary: Summary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pi: person::Summary,
}
impl Lab {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.summary.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.summary.name()
    }
}
