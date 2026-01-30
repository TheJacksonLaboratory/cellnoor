use std::sync::Arc;

use crate::{
    api::{auth, middleware::authenticate_request},
    db::{self},
    state::AppState,
};
use aide::{
    axum::{ApiRouter, routing::get},
    openapi::{OpenApi, SecurityScheme},
};
use axum::{Extension, Json, Router, extract::Request};
use schemars::JsonSchema;
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

// pub(super) mod cdna;
// pub(super) mod chromium_datasets;
// pub(super) mod chromium_runs;
// pub(super) mod gem_pools;
pub(super) mod institutions;
// pub(super) mod libraries;
// pub(super) mod multiplexing_tags;
pub(super) mod people;
pub(super) mod projects;
// pub(super) mod sequencing_runs;
pub(super) mod specimens;
pub(super) mod suspensions;
// pub(super) mod suspension_pools;
// pub(super) mod suspensions;
// pub(super) mod tenx_assays;

pub(super) fn app(state: AppState) -> Router<AppState> {
    let (router, api_docs) = router();

    let auth_layer = axum::middleware::from_fn_with_state(state, authenticate_request);
    let trace_layer = TraceLayer::new_for_http().make_span_with(
        |request: &Request| tracing::info_span!("http-request", uri = ?request.uri()),
    );

    let layers = ServiceBuilder::new().layer(auth_layer).layer(trace_layer);

    // Ensure the OpenAPI documentation is added after the authentication layer so it's public
    router
        .layer(layers)
        .route("/openapi.json", axum::routing::get(show_api_docs))
        .layer(Extension(Arc::new(api_docs)))
}

pub fn router() -> (Router<AppState>, OpenApi) {
    // This is the general error-type that can be a catch-all. If handlers return more specific errors, they can document that and associate it with status codes
    #[allow(dead_code)]
    #[derive(Serialize, JsonSchema)]
    #[serde(untagged)]
    enum ApiError {
        Auth(auth::Error),
        Database(db::Error),
    }

    aide::generate::infer_responses(true);

    let router = ApiRouter::new()
        .api_route("/health", get(async || "ok"))
        .nest("/institutions", institutions::router())
        .nest("/people", people::router())
        .nest("/projects", projects::router())
        .nest("/specimens", specimens::router());
    // .nest("/suspensions", suspensions::router());
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
            .version(env!("CARGO_PKG_VERSION"))
            .security_scheme(
                "api_token",
                SecurityScheme::Http {
                    scheme: "bearer".to_owned(),
                    bearer_format: Some("JWT".to_owned()),
                    description: None,
                    #[allow(clippy::default_trait_access)]
                    extensions: Default::default(),
                },
            )
            .security_requirement("api_token")
            .default_response::<Json<ApiError>>()
    });

    (router, api_docs)
}

async fn show_api_docs(Extension(api_docs): Extension<Arc<OpenApi>>) -> Json<Arc<OpenApi>> {
    Json(api_docs)
}
