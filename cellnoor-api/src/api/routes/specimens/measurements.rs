use aide::{
    axum::{ApiRouter, routing::get},
    transform::TransformOperation,
};
use axum::{Json, handler::Handler, http::StatusCode};

use crate::{admin_required_creation, state::AppState};

use create::create_specimen_measurement;
use index::index_specimen_measurements;

mod create;
mod index;

pub(super) fn router() -> ApiRouter<AppState> {
    ApiRouter::new().api_route(
        "/",
        get(index_specimen_measurements).post_with(
            create_specimen_measurement.layer(admin_required_creation!()),
            post_and_patch_docs,
        ),
    )
}

fn post_and_patch_docs(api_docs: TransformOperation) -> TransformOperation {
    api_docs.response::<{ StatusCode::UNPROCESSABLE_ENTITY.as_u16() }, Json<create::Error>>()
}
