use uuid::Uuid;

pub mod cdna;
pub mod chromium_runs;
pub mod index_sets;
pub mod institutions;
pub mod libraries;
pub mod multiplexing_tags;
pub mod people;
pub mod projects;
#[cfg(test)]
mod rls_tests;
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
