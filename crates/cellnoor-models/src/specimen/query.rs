#[cfg(feature = "app")]
use cellnoor_schema::specimens;
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use serde_json::Value;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::specimen::{
    Species,
    creation::block::BlockEmbeddingMatrix,
    variable::{Fixative, SpecimenType, ThermalPreservationMethod},
};

#[filter]
pub struct SpecimenFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
    pub submitted_by: Option<Vec<Uuid>>,
    pub project_ids: Option<Vec<Uuid>>,
    pub received_before: Option<Timestamp>,

    pub received_after: Option<Timestamp>,
    pub species: Option<Vec<Species>>,
    pub host_species: Option<Vec<Species>>,
    pub types: Option<Vec<SpecimenType>>,
    pub embedded_in: Option<Vec<BlockEmbeddingMatrix>>,
    pub fixatives: Option<Vec<Fixative>>,
    pub thermal_preservation_methods: Option<Vec<ThermalPreservationMethod>>,
    pub fresh: Option<bool>,
    pub tissues: Option<Vec<String>>,

    pub returned_before: Option<Timestamp>,

    pub returned_after: Option<Timestamp>,
    pub returned_by: Option<Vec<Uuid>>,
    pub additional_data: Option<Value>,
}

#[order_by(specimens)]
#[allow(non_camel_case_types)]
pub enum SpecimenOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    name { descending: Option<bool> },
    submitted_by { descending: Option<bool> },
    project_id { descending: Option<bool> },
    received_at { descending: Option<bool> },
    species { descending: Option<bool> },
    host_species { descending: Option<bool> },
    returned_at { descending: Option<bool> },
    returned_by { descending: Option<bool> },
    type_ { descending: Option<bool> },
    embedded_in { descending: Option<bool> },
    fixative { descending: Option<bool> },
    thermal_preservation_method { descending: Option<bool> },
    tissue { descending: Option<bool> },
}

impl Default for SpecimenOrderBy {
    fn default() -> Self {
        Self::received_at {
            descending: Some(true),
        }
    }
}

pub type SpecimenQuery = generic_query::Query<SpecimenFilter, SpecimenOrderBy>;
