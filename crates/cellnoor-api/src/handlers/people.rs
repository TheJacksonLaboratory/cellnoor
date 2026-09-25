use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub use create::create_person;
pub use index::index_people;
pub use show::show_person;
pub use update::update_person;

use crate::{
    db::DbError, error::error_response, handlers::specific_error_inferred_early_responses,
};

pub(super) mod create;
#[cfg(test)]
mod delete;
pub(super) mod index;
pub(super) mod show;
pub(super) mod update;

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PersonError {
    #[error("invalid email '{email}'")]
    InvalidEmail { email: String },
    #[serde(untagged)]
    #[error(transparent)]
    Db(#[from] DbError),
}

impl PersonError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidEmail { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Db(e) => e.status(),
        }
    }
}

impl IntoResponse for PersonError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for PersonError {
    type Inner = Self;

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        specific_error_inferred_early_responses::<Self>(ctx, operation)
    }
}
