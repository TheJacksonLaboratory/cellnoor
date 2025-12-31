#[cfg(feature = "app")]
use cellnoor_schema::specimens;
use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use serde_json::Value;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::specimen::common::{EmbeddingMatrix, Fixative, Species, SpecimenType};

#[filter]
pub struct SpecimenFilter {
    pub ids: Option<Vec<Uuid>>,
    pub names: Option<Vec<String>>,
    pub submitted_by: Option<Vec<Uuid>>,
    pub labs: Option<Vec<Uuid>>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub received_before: Option<Timestamp>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub received_after: Option<Timestamp>,
    pub species: Option<Vec<Species>>,
    pub host_species: Option<Vec<Species>>,
    pub types: Option<Vec<SpecimenType>>,
    pub embedded_in: Option<Vec<EmbeddingMatrix>>,
    pub fixatives: Option<Vec<Fixative>>,
    pub frozen: Option<bool>,
    pub cryopreserved: Option<bool>,
    pub tissues: Option<Vec<String>>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
    pub returned_before: Option<Timestamp>,
    #[cfg_attr(feature = "typescript", ts(as = "Option<String>"))]
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
    lab_id { descending: Option<bool> },
    received_at { descending: Option<bool> },
    species { descending: Option<bool> },
    host_species { descending: Option<bool> },
    returned_at { descending: Option<bool> },
    returned_by { descending: Option<bool> },
    type_ { descending: Option<bool> },
    embedded_in { descending: Option<bool> },
    fixative { descending: Option<bool> },
    frozen { descending: Option<bool> },
    cryopreserved { descending: Option<bool> },
    tissue { descending: Option<bool> },
}

impl Default for SpecimenOrderBy {
    fn default() -> Self {
        Self::received_at {
            descending: Some(true),
        }
    }
}

#[cfg(feature = "app")]
pub type SpecimenQuery = generic_query::Query<SpecimenFilter, SpecimenOrderBy>;

uuid_newtype!(SpecimenId, "/{id}");

uuid_newtype!(SpecimenIdMeasurements, "/{id}/measurements");

uuid_newtype!(SpecimenIdSuspensions, "/{id}/suspensions");

uuid_newtype!(SpecimenIdChromiumDatasets, "/{id}/chromium-datasets");
