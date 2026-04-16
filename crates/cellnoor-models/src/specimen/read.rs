#[cfg(feature = "app")]
use cellnoor_schema::{people, projects, specimens};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    person::PersonSummary,
    project::Project,
    specimen::{
        Species, common::SpecimenCommonFields, creation::block::BlockEmbeddingMatrix,
        variable::SpecimenVariableFields,
    },
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct SpecimenSummary {
    pub id: Uuid,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub links: SpecimenLinks,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub common: SpecimenCommonFields,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub variable: SpecimenVariableFields,
}

#[select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct SpecimenLinks {
    #[serde(rename = "self")]
    pub self_link: String,
    #[serde(rename = "measurements")]
    pub measurements_link: String,
    #[serde(rename = "suspensions")]
    pub suspensions_link: String,
    #[serde(rename = "chromium_datasets")]
    pub chromium_datasets_link: String,
}

impl SpecimenSummary {
    #[must_use]
    pub fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.common.name.as_ref()
    }

    #[must_use]
    pub fn received_at(&self) -> Timestamp {
        self.common.received_at
    }

    #[must_use]
    pub fn embedded_in(&self) -> Option<BlockEmbeddingMatrix> {
        self.variable.embedded_in
    }

    #[must_use]
    pub fn tissue(&self) -> &str {
        self.common.tissue.as_ref()
    }

    #[must_use]
    pub fn submitted_by(&self) -> Uuid {
        self.common.submitted_by
    }

    #[must_use]
    pub fn project_id(&self) -> Uuid {
        self.common.project_id
    }

    #[must_use]
    pub fn species(&self) -> Species {
        self.common.species
    }
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = specimens::table.inner_join(projects::table).inner_join(people::table.on(specimens::submitted_by.eq(people::id)))))]
pub struct Specimen {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    pub summary: SpecimenSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub project: Project,
    #[cfg_attr(feature = "app", diesel(embed))]
    pub submitted_by: PersonSummary,
}

impl Specimen {
    #[must_use]
    pub fn received_at(&self) -> Timestamp {
        self.summary.received_at()
    }
}
