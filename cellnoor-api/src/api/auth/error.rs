use aide::OperationIo;
use axum::{Json, http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;

use crate::db;

#[derive(Debug, thiserror::Error, serde::Serialize, JsonSchema)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "AuthError"))]
#[serde(rename_all = "snake_case", tag = "type")]
#[schemars(rename = "AuthError")]
#[error(transparent)]
pub enum Error {
    Database(#[from] db::Error),
    #[error("invalid auth token")]
    InvalidAuthToken,
    #[error("no auth token found")]
    NoAuthTokenFound {
        message: &'static str,
    },
    #[error("{message}")]
    Other {
        message: &'static str,
    },
    #[error("permission denied")]
    PermissionDenied,
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(_: jsonwebtoken::errors::Error) -> Self {
        Self::InvalidAuthToken
    }
}

impl Error {
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Database(_) | Self::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidAuthToken | Self::NoAuthTokenFound { .. } => StatusCode::UNAUTHORIZED,
            Self::PermissionDenied => StatusCode::FORBIDDEN,
        }
        .as_u16()
    }

    pub fn no_auth_token() -> Self {
        Self::NoAuthTokenFound {
            message: "no authorization token found in cookies nor in 'Authorization: Bearer' \
                      header",
        }
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for Error {
    fn from(err: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        Self::Database(err.into())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Self::Database(err.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(status = self.status_code(), error = %self);

        let status_code = self.status_code();
        let err = match self {
            Self::Database(..) => Self::Other {
                message: "something went wrong",
            },
            _ => self,
        };

        (StatusCode::from_u16(status_code).unwrap(), Json(err)).into_response()
    }
}
