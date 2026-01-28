use aide::transform::TransformOperation;
use axum::{Json, http::StatusCode};

use crate::{api::auth, db};

pub fn db_and_auth_error_docs(api: TransformOperation) -> TransformOperation {
    api.response::<{ StatusCode::UNPROCESSABLE_ENTITY.as_u16() }, Json<db::Error>>()
        .response::<{ StatusCode::CONFLICT.as_u16() }, Json<db::Error>>()
        .response::<{ StatusCode::INTERNAL_SERVER_ERROR.as_u16() }, Json<db::Error>>()
        .response::<{ StatusCode::NOT_FOUND.as_u16() }, Json<db::Error>>()
        .response::<{ StatusCode::UNAUTHORIZED.as_u16() }, Json<auth::Error>>()
        .response::<{ StatusCode::FORBIDDEN.as_u16() }, Json<auth::Error>>()
}
