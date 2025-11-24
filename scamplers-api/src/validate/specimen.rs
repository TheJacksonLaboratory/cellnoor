use jiff::Timestamp;
use scamplers_models::specimen::{Species, SpecimenCreation};

use crate::validate::Validate;

pub(super) mod measurement;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "cause")]
pub enum Error {
    #[error("donor and host species cannot be the same")]
    SameDonorAndHostSpecies { species: Species },
    #[error("received at ({received_at}) cannot be after returned at ({returned_at})")]
    ReturnedBeforeReceived {
        received_at: Timestamp,
        returned_at: Timestamp,
    },
}

impl Validate for SpecimenCreation {
    fn validate(&self, _db_conn: &mut diesel::PgConnection) -> Result<(), super::Error> {
        if let Some(host_species) = self.host_species()
            && host_species == self.species()
        {
            return Err(Error::SameDonorAndHostSpecies {
                species: host_species,
            })?;
        }

        if let Some(returned_at) = self.returned_at() {
            let received_at = self.received_at();

            if received_at >= returned_at {
                return Err(Error::ReturnedBeforeReceived {
                    received_at,
                    returned_at,
                })?;
            }
        }

        Ok(())
    }
}
