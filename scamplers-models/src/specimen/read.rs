#[cfg(feature = "app")]
use diesel::prelude::*;
use macro_attributes::select;
#[cfg(feature = "app")]
use scamplers_schema::{labs, people, specimens};

use crate::{
    lab::LabSummary,
    person::PersonSummary,
    specimen::common::{SpecimenCommonFields, SpecimenVariableFields},
};

#[select]
#[cfg_attr(feature = "app", diesel(table_name = specimens))]
pub struct SpecimenSummary {
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    common: SpecimenCommonFields,
    #[serde(flatten)]
    #[cfg_attr(feature = "app", diesel(embed))]
    variable: SpecimenVariableFields,
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
