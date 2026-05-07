use axum::{Json, extract::State};
use cellnoor_types::{ComplexQuery, Filter, SimpleQuery};
use schemars::JsonSchema;
use serde::Serialize;
use serde_qs::axum::QsQuery;

use crate::{auth::AuthUser, error::Error, state::AppState};

// pub mod cdna;
// pub mod chip_loadings;
// pub mod chromium_datasets;
// pub mod chromium_runs;
// pub mod database;
// pub mod gem_pools;
pub mod institutions;
// pub mod libraries;
// pub mod multiplexing_tags;
pub mod people;
pub mod projects;
pub mod specimens;
// pub mod suspension_pools;
// pub mod suspensions;
// pub mod tenx_assays;
