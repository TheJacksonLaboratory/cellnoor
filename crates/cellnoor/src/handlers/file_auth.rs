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
pub use dataset_dir::authorize_dataset_dir_access;
pub use project_dir::authorize_project_dir_access;

use crate::{db::DbError, error::error_response};

mod dataset_dir;
mod project_dir;

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileAuthError {
    #[error("cannot access this dataset")]
    DatasetAccessDenied,
    #[error("cannot access this project")]
    ProjectAccessDenied,
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl FileAuthError {
    fn status(&self) -> StatusCode {
        match self {
            Self::DatasetAccessDenied | Self::ProjectAccessDenied => StatusCode::UNAUTHORIZED,
            Self::Db(e) => e.status(),
        }
    }
}

impl IntoResponse for FileAuthError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for FileAuthError {
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
