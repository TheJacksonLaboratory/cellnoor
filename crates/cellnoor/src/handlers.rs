use uuid::Uuid;

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
