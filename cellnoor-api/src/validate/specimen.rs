use cellnoor_models::specimen::{Species, SpecimenCreation};
use jiff::Timestamp;

use crate::validate::{Validate, common::validate_timestamps};

pub(super) mod measurement;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "SpecimenValidationError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("donor and host species cannot be the same")]
    SameDonorAndHostSpecies { species: Species },
    #[error("received at ({received_at}) cannot be after returned at ({returned_at})")]
    ReturnedBeforeReceived {
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        received_at: Timestamp,
        #[cfg_attr(feature = "typescript", ts(as = "String"))]
        returned_at: Timestamp,
    },
}

impl Validate for SpecimenCreation {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        if let Some(host_species) = self.host_species() {
            validate_species(self.species(), host_species)?;
        }

        if let Some(returned_at) = self.returned_at() {
            validate_received_before_returned(self.received_at(), returned_at)?;
        }

        Ok(())
    }
}

fn validate_species(donor_species: Species, host_species: Species) -> Result<(), Error> {
    if donor_species == host_species {
        return Err(Error::SameDonorAndHostSpecies {
            species: host_species,
        });
    }

    Ok(())
}

fn validate_received_before_returned(
    received_at: Timestamp,
    returned_at: Timestamp,
) -> Result<(), super::Error> {
    validate_timestamps(received_at, returned_at, "returned_at")?;

    Ok(())
}
