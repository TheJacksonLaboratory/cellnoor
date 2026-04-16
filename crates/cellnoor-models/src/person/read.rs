#[cfg(feature = "app")]
use cellnoor_schema::{institutions, people};
#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
use uuid::Uuid;

use crate::{institution::Institution, person::common::PersonFields};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonSummary {
    pub id: Uuid,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: PersonFields,
    pub email: Option<String>,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: PersonLinks,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonLinks {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "projects")]
    pub projects_link: String,
    #[serde(rename = "specimens")]
    pub specimens_link: String,
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
#[cfg_attr(feature = "app", diesel(table_name = people))]
pub struct PersonSummaryStaff {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub inner: PersonSummary,
    pub microsoft_entra_oid: Option<Uuid>,
    pub is_admin: bool,
    pub is_biology_staff: bool,
    pub is_computational_staff: bool,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = people, base_query = people::table.inner_join(institutions::table)))]
pub struct Person {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub summary: PersonSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub institution: Institution,
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
