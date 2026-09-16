use uuid::Uuid;

use crate::{
    db::{self, AsFieldValuePairs, FieldValuePairs},
    error::ErrorInner,
};

pub mod accounts;
pub mod api_keys;
pub mod cdna;
pub mod chromium_datasets;
pub mod chromium_runs;
pub mod file_auth;
pub mod index_sets;
pub mod institutions;
pub mod libraries;
pub mod multiplexing_tags;
pub mod people;
pub(crate) mod permissions;
pub mod projects;
#[cfg(test)]
mod security_tests;
pub mod services;
pub mod specimens;
pub mod suspension_pools;
pub mod suspensions;
pub mod tenx_assays;

#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
    schemars::JsonSchema,
)]
#[schemars(inline)]
pub struct IdParam {
    pub id: Uuid,
}

struct IsStaff(bool);

impl AsFieldValuePairs<&'static str, 1> for IsStaff {
    fn as_field_value_pairs(&self) -> FieldValuePairs<'_, &'static str, 1> {
        [("is_staff", &self.0)]
    }
}

/// Set whether a person or service is staff.
///
/// `is_staff` lives on the `principal` table, whose row is created by a trigger
/// when the person or service is inserted, so it's always a separate statement.
pub(crate) async fn set_is_staff(
    tx: &db::Transaction<'_>,
    id: Uuid,
    is_staff: bool,
) -> Result<(), ErrorInner> {
    db::update(tx, "principal", id, &IsStaff(is_staff)).await
}
