#[cfg(feature = "app")]
use cellnoor_schema::{institutions, people};
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
use uuid::Uuid;

use crate::{institution::Institution, links::Links, person::common::PersonFields};

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
pub struct Person {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: PersonSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    institution: Institution,
}

impl Person {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.summary.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.summary.name()
    }

    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.summary.email()
    }
}
