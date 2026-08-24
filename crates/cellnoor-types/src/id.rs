use std::{fmt::Display, ops::Deref};

use macro_attributes::base_model;
#[cfg(feature = "postgres-types")]
use postgres_types::FromSql;
use uuid::Uuid;

// These structs allow us to use the same struct for inserting into and reading
// records from the db, so long as the model structs are parametrized. They are
// named `New` and `Saved` so that the resulting struct can be named
// `NewInstitution` or `SavedInstitution` in the resulting OpenAPI schema

#[base_model]
#[derive(Copy, Eq, Hash)]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct NoId;

#[base_model]
#[derive(Copy, Eq, Hash)]
#[cfg_attr(feature = "schemars", schemars(inline))]
pub struct Id {
    pub id: Uuid,
}

impl From<Uuid> for Id {
    fn from(id: Uuid) -> Self {
        Self { id }
    }
}

impl From<Id> for Uuid {
    fn from(id: Id) -> Self {
        id.id
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl Deref for Id {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

#[cfg(feature = "postgres-types")]
impl<'a> FromSql<'a> for Id {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let id = Uuid::from_sql(ty, raw)?;

        Ok(Self { id })
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        Uuid::accepts(ty)
    }
}
