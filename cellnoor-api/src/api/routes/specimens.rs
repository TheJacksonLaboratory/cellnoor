use crate::{
    api::middleware::{admin_required, created_status_code},
    state::AppState,
};
use aide::{
    axum::{
        ApiRouter,
        routing::{get, post},
    },
    transform::TransformOperation,
};
use axum::http::StatusCode;
use axum::{Json, handler::Handler};
use create::create_specimen;
use create_measurement::create_specimen_measurement;
use index::index_specimens;
use index_measurements::index_specimen_measurements;
use show::show_specimen;
use tower::ServiceBuilder;

pub(super) mod create;
pub(super) mod create_measurement;
pub(super) mod index;
pub(super) mod index_chromium_datasets;
pub(super) mod index_measurements;
pub(super) mod show;

pub(super) fn router() -> ApiRouter<AppState> {
    let creation_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(created_status_code));

    let id_router = ApiRouter::new()
        .api_route("/", get(show_specimen))
        .api_route(
            "/measurements",
            get(index_specimen_measurements).post_with(
                create_specimen_measurement.layer(creation_middleware.clone()),
                post_measurement_docs,
            ),
        )
        .api_route("/chromium-datasets", get(async || ()));

    let creation_middleware = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(admin_required))
        .layer(axum::middleware::map_response(created_status_code));

    ApiRouter::new()
        .api_route(
            "/",
            post(create_specimen.layer(creation_middleware)).get(index_specimens),
        )
        .nest("/{id}", id_router)
}

fn post_measurement_docs(api_docs: TransformOperation) -> TransformOperation {
    api_docs
        .response::<{ StatusCode::UNPROCESSABLE_ENTITY.as_u16() }, Json<create_measurement::Error>>(
        )
}
