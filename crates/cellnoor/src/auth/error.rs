use aide::OperationIo;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{db::DbError, error::error_response};

#[derive(
    Debug,
    Clone,
    thiserror::Error,
    serde::Serialize,
    schemars::JsonSchema,
    OperationIo,
    PartialEq,
    Eq,
)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthError {
    #[error("API key expired at {expired_at}")]
    ExpiredApiKey { expired_at: jiff::Timestamp },
    #[error("invalid auth token: {message}")]
    InvalidAuthToken { message: String },
    #[error("{message}")]
    NoAuthFound { message: &'static str },
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl AuthError {
    fn status(&self) -> StatusCode {
        match self {
            Self::ExpiredApiKey { .. }
            | Self::InvalidAuthToken { .. }
            | Self::NoAuthFound { .. } => StatusCode::UNAUTHORIZED,
            Self::Db(e) => e.status(),
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}
