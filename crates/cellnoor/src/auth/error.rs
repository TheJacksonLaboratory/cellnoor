use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{db::DbError, error::error_response};

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
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

impl OperationOutput for AuthError {
    type Inner = Self;

    fn operation_response(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Option<OpenApiResponse> {
        Json::<Self>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        let Some(response) = Self::operation_response(ctx, operation) else {
            return Vec::new();
        };

        vec![(None, response)]
    }
}
