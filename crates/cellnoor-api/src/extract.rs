use aide::{
    OperationInput, OperationIo,
    openapi::{MediaType, SchemaObject},
};
use axum::{extract::FromRequest, response::IntoResponse};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(FromRequest, Debug, Clone, PartialEq, Serialize)]
#[from_request(via(axum::Json), rejection(JsonRejection))]
pub struct JsonExtractor<T>(pub T);

impl<T: JsonSchema> OperationInput for JsonExtractor<T> {
    fn operation_input(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) {
        axum::Json::<T>::operation_input(ctx, operation);
    }

    fn inferred_early_responses(
        ctx: &mut aide::generate::GenContext,
        _operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<aide::openapi::StatusCode>, aide::openapi::Response)> {
        // Adapted from https://docs.rs/aide/0.16.0-alpha.4/src/aide/axum/inputs.rs.html#103
        let schema = SchemaObject {
            json_schema: ctx.schema.subschema_for::<JsonRejection>(),
            example: None,
            external_docs: None,
        };

        let response = |description: &'static str| aide::openapi::Response {
            description: description.to_owned(),
            content: [(
                "application/json".to_owned(),
                MediaType {
                    schema: Some(schema.clone()),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        // We only use 400 and 415 here and leave 422 for more specific errors.
        // See ./handlers/chromium_datasets/create.rs for an example
        vec![
            (
                Some(aide::openapi::StatusCode::Code(400)),
                response("failed to parse request body as JSON of the correct type"),
            ),
            (
                Some(aide::openapi::StatusCode::Code(415)),
                response("expected request with 'Content-Type: application/json'"),
            ),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema, OperationIo)]
#[aide(output_with = "axum::Json<JsonRejection>", json_schema)]
pub struct JsonRejection {
    status: u16,
    message: String,
}

impl From<axum::extract::rejection::JsonRejection> for JsonRejection {
    fn from(value: axum::extract::rejection::JsonRejection) -> Self {
        Self {
            status: value.status().as_u16(),
            message: value.body_text(),
        }
    }
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
