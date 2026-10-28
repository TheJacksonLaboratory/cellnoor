use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Log an error and convert it to an [`axum::response::Response`].
pub(crate) fn error_response<E>(status: StatusCode, error: E) -> Response
where
    E: serde::Serialize + std::fmt::Display,
{
    tracing::error!("{error}");

    (status, Json(error)).into_response()
}
