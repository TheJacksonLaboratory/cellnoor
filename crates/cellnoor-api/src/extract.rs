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
    fn inferred_early_responses(
        ctx: &mut aide::generate::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<aide::openapi::StatusCode>, aide::openapi::Response)> {
        let mut responses = axum::Json::<T>::inferred_early_responses(ctx, operation);
        for (_, resp) in &mut responses {
            let schema = SchemaObject {
                json_schema: ctx.schema.subschema_for::<JsonRejection>(),
                example: None,
                external_docs: None,
            };

            resp.content = [(
                "application/json".to_owned(),
                MediaType {
                    schema: Some(schema),
                    ..Default::default()
                },
            )]
            .into_iter()
            .collect();
        }

        responses
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
