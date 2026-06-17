use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::{ApiKeyLocation, OpenApi, SecurityScheme},
    redoc::Redoc,
};
use axum::{Extension, Json, Router, routing::get};

use crate::state::AppState;

mod accounts;
mod api_keys;
mod cdna;
mod chromium_datasets;
mod chromium_runs;
pub(super) mod file_auth;
mod index_sets;
mod institutions;
mod libraries;
mod multiplexing_tags;
mod people;
mod projects;
mod services;
mod specimens;
mod suspension_pools;
mod suspensions;
mod tenx_assays;

pub fn router() -> (OpenApi, Router<AppState>) {
    let router = ApiRouter::new()
        .route("/health", get(async || "ok"))
        .route("/openapi.json", aide::axum::routing::get(show_api_docs))
        .route(
            "/docs/redoc",
            get(Redoc::new("/api/openapi.json")
                .with_title("cellnoor RESTful API")
                .axum_handler()),
        )
        .nest("/institutions", institutions::router())
        .nest("/people", people::router())
        .nest("/accounts", accounts::router())
        .nest("/service-accounts", services::router())
        .nest("/api-keys", api_keys::router())
        .nest("/projects", projects::router())
        .nest("/index-sets", index_sets::router())
        .nest("/10x-assays", tenx_assays::router())
        .nest("/multiplexing-tags", multiplexing_tags::router())
        .nest("/specimens", specimens::router())
        .nest("/suspensions", suspensions::router())
        .nest("/suspension-pools", suspension_pools::router())
        .nest("/chromium-runs", chromium_runs::router())
        .nest("/cdna", cdna::router())
        .nest("/libraries", libraries::router())
        .nest("/chromium-datasets", chromium_datasets::router());

    let mut api_docs = OpenApi::default();

    let router = router
        .finish_api_with(&mut api_docs, |api_docs| {
            api_docs
                .title("cellnoor RESTful API")
                .version("0.1.0")
                .security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey {
                        location: ApiKeyLocation::Header,
                        name: "x-api-key".to_owned(),
                        extensions: Default::default(),
                        description: None,
                    },
                )
                .security_requirement("api_key")
        })
        .layer(Extension(Arc::new(api_docs.clone())));

    (api_docs, router)
}

#[axum::debug_handler]
async fn show_api_docs(Extension(api_docs): Extension<Arc<OpenApi>>) -> Json<Arc<OpenApi>> {
    Json(api_docs)
}
