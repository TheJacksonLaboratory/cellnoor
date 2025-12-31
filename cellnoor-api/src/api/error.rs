use axum::{
    Json,
    extract::{
        multipart::MultipartError,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::IntoResponse,
};

use crate::{api::extract::auth, db, validate};

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "ApiError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error(transparent)]
pub enum Error {
    Auth(#[from] auth::Error),
    Data(#[from] validate::Error),
    Database(#[from] db::Error),
    #[error("{message}")]
    MalformedRequest {
        message: String,
    },
    #[error("something went wrong")]
    Other,
}

impl From<deadpool_diesel::InteractError> for Error {
    fn from(err: deadpool_diesel::InteractError) -> Self {
        Self::Database(err.into())
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Self::MalformedRequest {
            message: format!("failed to parse CSV: {err}"),
        }
    }
}

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "ApiErrorResponse"))]
#[error("{self:?}")]
pub struct ErrorResponse {
    pub status: u16,
    #[serde(flatten)]
    pub public_error: Error,
    #[serde(skip)]
    pub internal_error: Option<Error>,
}

impl From<JsonRejection> for ErrorResponse {
    fn from(err: JsonRejection) -> Self {
        Self {
            status: err.status().as_u16(),
            public_error: Error::MalformedRequest {
                message: err.body_text(),
            },
            internal_error: None,
        }
    }
}

impl From<PathRejection> for ErrorResponse {
    fn from(err: PathRejection) -> Self {
        Self {
            status: err.status().as_u16(),
            public_error: Error::MalformedRequest {
                message: err.body_text(),
            },
            internal_error: None,
        }
    }
}

impl From<MultipartError> for ErrorResponse {
    fn from(err: MultipartError) -> Self {
        Self {
            status: err.status().as_u16(),
            public_error: Error::MalformedRequest {
                message: err.body_text(),
            },
            internal_error: None,
        }
    }
}

impl From<serde_qs::axum::QsQueryRejection> for ErrorResponse {
    fn from(err: serde_qs::axum::QsQueryRejection) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
            public_error: Error::MalformedRequest {
                message: err.to_string(),
            },
            internal_error: None,
        }
    }
}

impl From<deadpool_diesel::InteractError> for ErrorResponse {
    fn from(err: deadpool_diesel::InteractError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            public_error: Error::from(err),
            internal_error: None,
        }
    }
}

impl From<auth::Error> for ErrorResponse {
    fn from(err: auth::Error) -> Self {
        use auth::Error::{Database, Unauthorized};
        match err {
            Unauthorized { .. } => Self {
                status: StatusCode::UNAUTHORIZED.as_u16(),
                public_error: err.into(),
                internal_error: None,
            },
            Database(e) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                public_error: Error::Other,
                internal_error: Some(e.into()),
            },
        }
    }
}

impl From<db::Error> for ErrorResponse {
    fn from(err: db::Error) -> Self {
        use db::Error::{Data, DuplicateResource, InvalidReference, Other, ResourceNotFound};
        let status = match err {
            DuplicateResource { .. } => StatusCode::CONFLICT,
            Data { .. } | InvalidReference { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ResourceNotFound { .. } => StatusCode::NOT_FOUND,
            Other { .. } => {
                return {
                    Self {
                        status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                        public_error: Error::Other,
                        internal_error: Some(err.into()),
                    }
                };
            }
        };

        Self {
            status: status.as_u16(),
            public_error: Error::Database(err),
            internal_error: None,
        }
    }
}

impl From<validate::Error> for ErrorResponse {
    fn from(err: validate::Error) -> Self {
        match err {
            validate::Error::Database(e) => Self::from(e),
            err => Self {
                status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                public_error: err.into(),
                internal_error: None,
            },
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{self}");
        (StatusCode::from_u16(self.status).unwrap(), Json(self)).into_response()
    }
}
