use axum::{
    Json,
    extract::{
        multipart::MultipartError,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::IntoResponse,
};

use super::{auth, routes};
use crate::db;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error(transparent)]
pub enum DataError {
    CreatePerson(#[from] super::routes::people::CreatePersonError),
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "ApiError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error(transparent)]
pub enum Error {
    Auth(#[from] auth::Error),
    Data(#[from] DataError),
    Database(#[from] db::Error),
    #[error("{message}")]
    MalformedRequest {
        message: String,
    },
    #[error("something went wrong")]
    Other,
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for Error {
    fn from(err: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        Self::Database(err.into())
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::MalformedRequest {
            message: format!("failed to parse CSV: {err}"),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Self::Database(err.into())
    }
}

impl From<JsonRejection> for Error {
    fn from(err: JsonRejection) -> Self {
        Self::MalformedRequest {
            message: err.body_text(),
        }
    }
}

impl From<PathRejection> for Error {
    fn from(err: PathRejection) -> Self {
        Self::MalformedRequest {
            message: err.body_text(),
        }
    }
}

impl From<MultipartError> for Error {
    fn from(err: MultipartError) -> Self {
        Self::MalformedRequest {
            message: err.body_text(),
        }
    }
}

impl From<serde_qs::axum::QsQueryRejection> for Error {
    fn from(err: serde_qs::axum::QsQueryRejection) -> Self {
        Self::MalformedRequest {
            message: err.to_string(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            error: Error,
        }

        tracing::error!(error = ?self);

        let (status_code, error) = match self {
            Self::Auth(auth::Error::Database(_))
            | Self::Database(db::Error::Other { .. })
            | Self::Other => (StatusCode::INTERNAL_SERVER_ERROR, Self::Other),
            Self::Auth(
                auth::Error::InvalidAuthToken { .. } | auth::Error::NoAuthTokenFound { .. },
            ) => (StatusCode::INTERNAL_SERVER_ERROR, self),
            Self::Auth(auth::Error::PermissionDenied) => (StatusCode::FORBIDDEN, self),
            Self::Data(_)
            | Self::MalformedRequest { .. }
            | Self::Database(db::Error::Data { .. })
            | Self::Database(db::Error::InvalidReference { .. }) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self)
            }
            Self::Database(db::Error::DuplicateResource { .. }) => (StatusCode::CONFLICT, self),
            Self::Database(db::Error::ResourceNotFound { .. }) => (StatusCode::NOT_FOUND, self),
        };

        (status_code, Json(ErrorResponse { error })).into_response()
    }
}
