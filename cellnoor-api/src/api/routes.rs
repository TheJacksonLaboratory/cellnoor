use axum::{Json, Router, extract::State, http::StatusCode};
use diesel::Connection;

use crate::{api::extract::auth::AuthenticatedUser, db, state::AppState};

// pub(super) mod cdna;
// pub(super) mod chromium_datasets;
// pub(super) mod chromium_runs;
// pub(super) mod gem_pools;
pub(super) mod institutions;
// pub(super) mod libraries;
// pub(super) mod multiplexing_tags;
// pub(super) mod people;
// pub(super) mod projects;
// pub(super) mod sequencing_runs;
// pub(super) mod specimens;
// pub(super) mod suspension_pools;
// pub(super) mod suspensions;
// pub(super) mod tenx_assays;

pub(super) fn router() -> Router<AppState> {
    Router::new().merge(institutions::router())
    // .merge(people::router())
    // .nest("/projects", projects::router())
    // .nest("/specimens", specimens::router())
    // .nest("/10x-assays", tenx_assays::router())
    // .nest("/sequencing-runs", sequencing_runs::router())
    // .nest("/multiplexing-tags", multiplexing_tags::router())
    // .nest("/suspensions", suspensions::router())
    // .nest("/suspension-pools", suspension_pools::router())
    // .nest("/chromium-runs", chromium_runs::router())
    // .nest("/gem-pools", gem_pools::router())
    // .nest("/cdna", cdna::router())
    // .nest("/libraries", libraries::router())
    // .nest("/chromium-datasets", chromium_datasets::router())
}
