use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    extract::multipart::MultipartError,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    db::DbError, error::error_response, handlers::specific_error_inferred_early_responses,
};

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UploadFilesError {
    #[error("failed to read multipart form: {message}")]
    Multipart { message: String },
    #[error("{message}")]
    ContentType { message: String },
    #[error("{message}")]
    Filename { message: String },
    #[error("failed to parse uploaded file: {message}")]
    Parse { message: String },
    #[error("failed to write file to disk: {message}")]
    Write { message: String },
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl UploadFilesError {
    fn status(&self) -> StatusCode {
        match self {
            Self::Multipart { .. }
            | Self::ContentType { .. }
            | Self::Filename { .. }
            | Self::Parse { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Db(e) => e.status(),
            Self::Write { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<MultipartError> for UploadFilesError {
    fn from(value: MultipartError) -> Self {
        Self::Multipart {
            message: value.to_string(),
        }
    }
}

impl From<std::io::Error> for UploadFilesError {
    fn from(value: std::io::Error) -> Self {
        Self::Write {
            message: value.to_string(),
        }
    }
}

impl IntoResponse for UploadFilesError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for UploadFilesError {
    type Inner = Self;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        specific_error_inferred_early_responses::<Self>(ctx, operation)
    }
}
