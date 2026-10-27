use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{db::DbError, error::error_response, handlers::specific_error_inferred_early_responses};

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemovePreparersError {
    #[error("at least one preparer must remain")]
    NoPreparersLeft,
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl RemovePreparersError {
    fn status(&self) -> StatusCode {
        match self {
            Self::NoPreparersLeft => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Db(e) => e.status(),
        }
    }
}

impl IntoResponse for RemovePreparersError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for RemovePreparersError {
    type Inner = Self;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        specific_error_inferred_early_responses::<Self>(ctx, operation)
    }
}
