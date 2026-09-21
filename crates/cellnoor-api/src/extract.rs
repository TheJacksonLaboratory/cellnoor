use aide::{OperationInput, OperationOutput, generate::GenContext};
use axum::{extract::FromRequest, response::IntoResponse};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(FromRequest, Debug, Clone, PartialEq, Serialize)]
#[from_request(via(axum::Json), rejection(JsonRejection))]
pub struct JsonExtractor<T>(pub T);

impl From<axum::extract::rejection::JsonRejection> for JsonRejection {
    fn from(value: axum::extract::rejection::JsonRejection) -> Self {
        Self {
            status: value.status().as_u16(),
            message: value.body_text(),
        }
    }
}

impl<T: JsonSchema> OperationInput for JsonExtractor<T> {
    fn inferred_early_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<aide::openapi::StatusCode>, aide::openapi::Response)> {
        axum::Json::<T>::inferred_early_responses(ctx, operation)
    }

    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        axum::Json::<T>::operation_input(ctx, operation);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct JsonRejection {
    status: u16,
    message: String,
}

impl IntoResponse for JsonRejection {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::from_u16(self.status).unwrap(),
            axum::extract::Json(self).into_response(),
        )
            .into_response()
    }
}
impl OperationOutput for JsonRejection {
    type Inner = Self;

    fn operation_response(
        ctx: &mut GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        axum::Json::<Self>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<aide::openapi::StatusCode>, aide::openapi::Response)> {
        let Some(response) = Self::operation_response(ctx, operation) else {
            return Vec::new();
        };

        vec![(None, response)]
    }
}
