use std::{fmt::Debug, sync::Arc};

use crate::{api::middleware::authenticate_request, db::DbConnection, state::AppState};
use aide::{
    axum::{ApiRouter, AxumOperationHandler, IntoApiResponse, routing::get},
    openapi::{OpenApi, SecurityScheme},
};
use axum::{
    Extension, Json, Router,
    error_handling::HandleErrorLayer,
    extract::{FromRequest, FromRequestParts, Request},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use diesel::Connection;
use serde::{Serialize, de::DeserializeOwned};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info_span;

// pub(super) mod cdna;
// pub(super) mod chromium_datasets;
// pub(super) mod chromium_runs;
// pub(super) mod gem_pools;
pub(super) mod institutions;
// pub(super) mod libraries;
// pub(super) mod multiplexing_tags;
pub(super) mod people;
// pub(super) mod projects;
// pub(super) mod sequencing_runs;
// pub(super) mod specimens;
// pub(super) mod suspension_pools;
// pub(super) mod suspensions;
// pub(super) mod tenx_assays;

pub(super) fn router(state: AppState) -> Router<AppState> {
    aide::generate::infer_responses(true);

    let router = ApiRouter::new()
        .api_route("/health", get(async || "ok"))
        .nest("/institutions", institutions::router())
        .nest("/people", people::router());
    // .merge(projects::router());
    // .merge(specimens::router())
    // .merge(tenx_assays::router())
    // .merge(sequencing_runs::router())
    // .merge(multiplexing_tags::router())
    // .merge(suspensions::router())
    // .merge(suspension_pools::router())
    // .merge(chromium_runs::router())
    // .merge(gem_pools::router())
    // .merge(cdna::router())
    // .merge(libraries::router())
    // .merge(chromium_datasets::router())

    let mut api_docs = OpenApi::default();

    let router = router.finish_api_with(&mut api_docs, |api_docs| {
        api_docs
            .title("cellnoor REST API")
            .version("0.1.0")
            .security_scheme(
                "api_token",
                SecurityScheme::Http {
                    scheme: "bearer".to_owned(),
                    bearer_format: Some("JWT".to_owned()),
                    description: None,
                    extensions: Default::default(),
                },
            )
            .security_requirement("api_token")
    });

    let layers = ServiceBuilder::new()
        .layer(axum::middleware::from_fn_with_state(
            state,
            authenticate_request,
        ))
        .layer(TraceLayer::new_for_http().make_span_with(
            |request: &Request| tracing::info_span!("http-request", uri = ?request.uri()),
        ));

    // By adding OpenAPI documentation route after the layers, it won't require authentication
    router
        .layer(layers)
        .route("/openapi.json", axum::routing::get(show_api_docs))
        .layer(Extension(Arc::new(api_docs)))
}

#[axum::debug_handler]
async fn show_api_docs(Extension(api_docs): Extension<Arc<OpenApi>>) -> Json<Arc<OpenApi>> {
    Json(api_docs)
}
