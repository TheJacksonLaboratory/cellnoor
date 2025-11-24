use jiff::Timestamp;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
#[cfg(feature = "app")]
use scamplers_schema::specimens;
use serde_json::Value;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;
use crate::specimen::common::{EmbeddingMatrix, Fixative, Species, SpecimenType};

#[filter]
pub struct SpecimenFilter {
    ids: Option<Vec<Uuid>>,
    names: Option<Vec<String>>,
    submitted_by: Option<Vec<Uuid>>,
    labs: Option<Vec<Uuid>>,
    #[cfg_attr(feature = "typescript", ts(type = "Date"))]
    received_before: Option<Timestamp>,
    #[cfg_attr(feature = "typescript", ts(type = "Date"))]
    received_after: Option<Timestamp>,
    species: Option<Vec<Species>>,
    host_species: Option<Vec<Species>>,
    types: Option<Vec<SpecimenType>>,
    embedded_in: Option<Vec<EmbeddingMatrix>>,
    fixatives: Option<Vec<Fixative>>,
    frozen: Option<bool>,
    cryopreserved: Option<bool>,
    tissues: Option<Vec<String>>,
    #[cfg_attr(feature = "typescript", ts(type = "Date"))]
    returned_before: Option<Timestamp>,
    #[cfg_attr(feature = "typescript", ts(type = "Date"))]
    returned_after: Option<Timestamp>,
    returned_by: Option<Vec<Uuid>>,
    additional_data: Option<Vec<Value>>,
}

impl SpecimenFilter {
    #[must_use]
    pub fn ids(&self) -> Option<&[Uuid]> {
        self.ids.as_deref()
    }

    #[must_use]
    pub fn names(&self) -> Option<&[String]> {
        self.names.as_deref()
    }

    #[must_use]
    pub fn submitted_by(&self) -> Option<&[Uuid]> {
        self.submitted_by.as_deref()
    }

    #[must_use]
    pub fn labs(&self) -> Option<&[Uuid]> {
        self.labs.as_deref()
    }

    #[must_use]
    pub fn received_before(&self) -> Option<Timestamp> {
        self.received_before
    }

    #[must_use]
    pub fn received_after(&self) -> Option<Timestamp> {
        self.received_after
    }

    #[must_use]
    pub fn species(&self) -> Option<&[Species]> {
        self.species.as_deref()
    }

    #[must_use]
    pub fn host_species(&self) -> Option<&[Species]> {
        self.host_species.as_deref()
    }

    #[must_use]
    pub fn types(&self) -> Option<&[SpecimenType]> {
        self.types.as_deref()
    }

    #[must_use]
    pub fn embedded_in(&self) -> Option<&[EmbeddingMatrix]> {
        self.embedded_in.as_deref()
    }

    #[must_use]
    pub fn fixatives(&self) -> Option<&[Fixative]> {
        self.fixatives.as_deref()
    }

    #[must_use]
    pub fn frozen(&self) -> Option<bool> {
        self.frozen
    }

    #[must_use]
    pub fn cryopreserved(&self) -> Option<bool> {
        self.cryopreserved
    }

    #[must_use]
    pub fn tissues(&self) -> Option<&[String]> {
        self.tissues.as_deref()
    }

    #[must_use]
    pub fn returned_by(&self) -> Option<&[Uuid]> {
        self.submitted_by.as_deref()
    }

    #[must_use]
    pub fn returned_before(&self) -> Option<Timestamp> {
        self.returned_before
    }

    #[must_use]
    pub fn returned_after(&self) -> Option<Timestamp> {
        self.returned_after
    }

    #[must_use]
    pub fn additional_data(&self) -> Option<&[Value]> {
        self.additional_data.as_deref()
    }
}

#[order_by(specimens)]
#[allow(non_camel_case_types)]
pub enum SpecimenOrderBy {
    id { descending: bool },
    name { descending: bool },
    readable_id { descending: bool },
    received_at { descending: bool },
}

impl Default for SpecimenOrderBy {
    fn default() -> Self {
        Self::received_at { descending: false }
    }
}

#[cfg(feature = "app")]
pub type SpecimenQuery = generic_query::Query<SpecimenFilter, SpecimenOrderBy>;

uuid_newtype!(SpecimenId, "/{id}");

uuid_newtype!(SpecimenIdMeasurements, "/{id}/measurements");

uuid_newtype!(SpecimenIdSuspensions, "/{id}/suspensions");

uuid_newtype!(SpecimenIdChromiumDatasets, "/{id}/chromium-datasets");
