#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::{base_model, select};
#[cfg(feature = "app")]
use scamplers_schema::{institutions, people};
use uuid::Uuid;

use crate::{
    institution::Institution,
    links::Links,
    person::{UserRole, common::Fields},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = people))]
#[cfg_attr(feature = "typescript", ts(rename = "PersonSummary"))]
pub struct Summary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: Fields,
    email: Option<String>,
    email_verified: bool,
    #[cfg_attr(feature = "typescript", ts(inline))]
    links: Links,
}

impl Summary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.inner.name.as_ref()
    }

    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = people, base_query = people::table.inner_join(institutions::table)))]
pub struct SummaryWithParents {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: Summary,
    #[cfg_attr(feature = "app", diesel(embed))]
    institution: Institution,
}

#[base_model]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct Person {
    #[serde(flatten)]
    info: SummaryWithParents,
    roles: Vec<UserRole>,
}
impl Person {
    #[must_use]
    pub fn new(info: SummaryWithParents, roles: Vec<UserRole>) -> Self {
        Self { info, roles }
    }

    #[must_use]
    pub fn id(&self) -> Uuid {
        self.info.summary.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.info.summary.name()
    }

    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.info.summary.email()
    }
}
