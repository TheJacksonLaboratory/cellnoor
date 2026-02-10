use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::{OpenApi, SecurityScheme},
};
use axum::{Extension, Json, Router, extract::Request, routing::get};
use schemars::JsonSchema;
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    api::{auth, middleware::authenticate_request},
    db::{self},
    state::AppState,
};

pub mod cdna;
pub mod chromium_datasets;
pub mod chromium_runs;
pub mod gem_pools;
pub mod institutions;
pub mod libraries;
pub mod multiplexing_tags;
pub mod people;
pub mod projects;
pub mod sequencing_runs;
pub mod specimens;
pub mod suspension_pools;
pub mod suspensions;
pub mod tenx_assays;

pub(super) fn app(state: AppState) -> Router<AppState> {
    let (router, api_docs) = router();

    let auth_layer = axum::middleware::from_fn_with_state(state, authenticate_request);
    let trace_layer = TraceLayer::new_for_http().make_span_with(
        |request: &Request| tracing::info_span!("http-request", uri = ?request.uri()),
    );

    let layers = ServiceBuilder::new().layer(auth_layer).layer(trace_layer);

    // Ensure the OpenAPI documentation route and health route are added after the
    // authentication layer so they're public
    router
        .layer(layers)
        .route("/health", get(async || "ok"))
        .route("/openapi.json", axum::routing::get(show_api_docs))
        .layer(Extension(Arc::new(api_docs)))
}

pub fn router() -> (Router<AppState>, OpenApi) {
    // This is the general error-type that can be a catch-all. If handlers return
    // more specific errors, they can document that and associate it with status
    // codes
    #[allow(dead_code)]
    #[derive(Serialize, JsonSchema)]
    #[serde(untagged)]
    enum ApiError {
        Auth(auth::Error),
        Database(db::Error),
    }

    aide::generate::infer_responses(true);

    let router = ApiRouter::new()
        .nest("/institutions", institutions::router())
        .nest("/people", people::router())
        .nest("/projects", projects::router())
        .nest("/specimens", specimens::router())
        .nest("/10x-assays", tenx_assays::router())
        .nest("/sequencing-runs", sequencing_runs::router())
        .nest("/multiplexing-tags", multiplexing_tags::router())
        .nest("/suspensions", suspensions::router())
        .nest("/suspension-pools", suspension_pools::router())
        .nest("/chromium-runs", chromium_runs::router())
        .nest("/gem-pools", gem_pools::router())
        .nest("/cdna", cdna::router())
        .nest("/libraries", libraries::router())
        .nest("/chromium-datasets", chromium_datasets::router());

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
