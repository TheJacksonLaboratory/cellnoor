#[cfg(feature = "app")]
use cellnoor_schema::{labs, people, specimens};
#[cfg(feature = "app")]
use diesel::prelude::*;
use jiff::Timestamp;
use macro_attributes::select;
use uuid::Uuid;

use crate::{
    lab::LabSummary,
    links::Links,
    person::PersonSummary,
    specimen::common::{EmbeddingMatrix, SpecimenCommonFields, SpecimenVariableFields},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct SpecimenSummary {
    id: Uuid,
    links: Links,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    common: SpecimenCommonFields,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    variable: SpecimenVariableFields,
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
    pub fn embedded_in(&self) -> Option<EmbeddingMatrix> {
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
}

#[select]
#[cfg_attr(feature = "app", diesel(base_query = specimens::table.inner_join(labs::table).inner_join(people::table.on(specimens::submitted_by.eq(people::id)))))]
pub struct Specimen {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    summary: SpecimenSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    lab: LabSummary,
    #[cfg_attr(feature = "app", diesel(embed))]
    submitted_by: PersonSummary,
}

impl Specimen {
    #[must_use]
    pub fn received_at(&self) -> Timestamp {
        self.summary.received_at()
    }
}
