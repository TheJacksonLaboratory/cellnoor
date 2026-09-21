use aide::OperationIo;
use axum::{extract::FromRequest, response::IntoResponse};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, OperationIo, Serialize)]
pub struct JsonRejection {
    status: u16,
    message: String,
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(JsonRejection))]
pub struct Json<T>(T);

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
            axum::Json(self).into_response(),
        )
            .into_response()
    }
}
