use cellnoor_types::Relation;
use uuid::Uuid;

use crate::{
    db::{self, FieldValues, Insert},
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

impl Relation for IsStaff {
    const NAME: &'static str = "principal";
}

async fn f() {
    use cellnoor_types::institution::SavedInstitutionRecord;
    let x: deadpool_postgres::Client = todo!();
    let y: SavedInstitutionRecord = x
        .query_one_scalar("select institution from institution", &[])
        .await
        .unwrap();
}

impl Insert for IsStaff {
    type Field = &'static str;

    fn fields(&self) -> FieldValues<'_, Self::Field> {
        vec![("is_staff", &self.0)]
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
    tx.update(id, &IsStaff(is_staff)).await
}
