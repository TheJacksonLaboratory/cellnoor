#[cfg(feature = "app")]
use cellnoor_schema::cdna;
use macro_attributes::{filter, order_by};
use macros::uuid_newtype;
use uuid::Uuid;

#[cfg(feature = "app")]
use crate::generic_query;

#[filter]
pub struct CdnaFilter {
    pub ids: Option<Vec<Uuid>>,
}

#[order_by(cdna)]
#[allow(non_camel_case_types)]
pub enum CdnaOrderBy {
    id { descending: Option<bool> },
    readable_id { descending: Option<bool> },
    library_type { descending: Option<bool> },
    prepared_at { descending: Option<bool> },
    gem_pool_id { descending: Option<bool> },
    n_amplification_cycles { descending: Option<bool> },
}

impl Default for CdnaOrderBy {
    fn default() -> Self {
        Self::prepared_at {
            descending: Some(true),
        }
    }
}

#[cfg(feature = "app")]
pub type CdnaQuery = generic_query::Query<CdnaFilter, CdnaOrderBy>;

uuid_newtype!(CdnaId, "/{id}");

uuid_newtype!(CdnaIdMeasurements, "/{id}/measurements");
