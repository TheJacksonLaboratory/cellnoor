#[cfg(feature = "app")]
use cellnoor_schema::{institutions, people};
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::{base_model, select};
use uuid::Uuid;

use crate::{
    institution::Institution,
    links::Links,
    person::{UserRole, common::PersonFields},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonSummary {
    id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    inner: PersonFields,
    email: Option<String>,
    email_verified: bool,
    links: Links,
}

impl PersonSummary {
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
pub struct PersonSummaryWithParents {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: PersonSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    institution: Institution,
}

#[base_model]
#[derive(serde::Serialize)]
#[cfg_attr(feature = "builder", derive(bon::Builder))]
pub struct Person {
    #[serde(flatten)]
    info: PersonSummaryWithParents,
    roles: Vec<UserRole>,
}
impl Person {
    #[must_use]
    pub fn new(info: PersonSummaryWithParents, roles: Vec<UserRole>) -> Self {
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
