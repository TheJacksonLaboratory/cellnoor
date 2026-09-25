use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub use dataset_dir::authorize_dataset_dir_access;
pub use project_dir::authorize_project_dir_access;

use crate::{
    db::DbError, error::error_response, handlers::specific_error_inferred_early_responses,
};

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

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        let mut responses = specific_error_inferred_early_responses::<Self>(ctx, operation);
        let (_, response) = responses.swap_remove(0);

        vec![(Some(aide::openapi::StatusCode::Code(401)), response)]
    }
}
