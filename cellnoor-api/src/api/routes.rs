use std::fmt::Debug;

use crate::{
    api::{
        self,
        extract::{Json, Path, PathAndJson, PathAndQuery, QsQuery},
        request::handle_api_request,
    },
    state::AppState,
};
use axum::{
    Router,
    extract::{FromRequest, FromRequestParts},
    http::StatusCode,
};
use axum_extra::routing::Resource;
use diesel::Connection;
use serde::{Serialize, de::DeserializeOwned};

pub(super) mod cdna;
pub(super) mod chromium_datasets;
pub(super) mod chromium_runs;
pub(super) mod gem_pools;
pub(super) mod institutions;
pub(super) mod libraries;
pub(super) mod multiplexing_tags;
pub(super) mod people;
pub(super) mod projects;
pub(super) mod sequencing_runs;
pub(super) mod specimens;
pub(super) mod suspension_pools;
pub(super) mod suspensions;
pub(super) mod tenx_assays;

pub(super) fn router() -> Router<AppState> {
    Router::new()
        .merge(institutions::router())
        .merge(people::router())
        .merge(projects::router())
        .merge(specimens::router())
        .merge(tenx_assays::router())
        .merge(sequencing_runs::router())
        .merge(multiplexing_tags::router())
        .merge(suspensions::router())
        .merge(suspension_pools::router())
        .merge(chromium_runs::router())
        .merge(gem_pools::router())
        .merge(cdna::router())
        .merge(libraries::router())
        .merge(chromium_datasets::router())
}
