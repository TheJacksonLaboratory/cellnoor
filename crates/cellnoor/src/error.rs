use aide::OperationIo;
use axum::{Json, http::StatusCode, response::IntoResponse};
use deadpool_postgres::{
    PoolError as DeadpoolPgError,
    tokio_postgres::{Error as TokioPgError, error::SqlState},
};

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, OperationIo)]
#[error(transparent)]
pub struct Error {
    error: ErrorInner,
}

impl Error {
    pub fn resource_not_found() -> Self {
        Self {
            error: ErrorInner::ResourceNotFound,
        }
    }

    pub fn no_auth_found(message: &'static str) -> Self {
        Self {
            error: ErrorInner::NoAuthFound { message },
        }
    }

    pub fn invalid_api_key() -> Self {
        Self {
            error: ErrorInner::InvalidApiKey,
        }
    }

    pub fn expired_api_key(expired_at: jiff::Timestamp) -> Self {
        Self {
            error: ErrorInner::ExpiredApiKey { expired_at },
        }
    }

    pub fn other(message: String) -> Self {
        Self {
            error: ErrorInner::Other { message },
        }
    }
}

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ErrorInner {
    #[error("resource not found")]
    ResourceNotFound,
    #[error("invalid API key")]
    InvalidApiKey,
    #[error("API key expired at {expired_at}")]
    ExpiredApiKey { expired_at: jiff::Timestamp },
    #[error("invalid auth token")]
    InvalidAuthToken,
    #[error("{message}")]
    NoAuthFound { message: &'static str },
    #[error("invalid data in field {} for {} - {} ({})", field.clone().unwrap_or_default(), resource.clone().unwrap_or_default(), message, detail.clone().unwrap_or_default())]
    DataConstraint {
        resource: Option<String>,
        field: Option<String>,
        message: String,
        detail: Option<String>,
    },
    #[error("permission denied")]
    PermissionDenied,
    #[error("{message}")]
    Other { message: String },
}

impl IntoResponse for Error {
    fn into_response(mut self) -> axum::response::Response {
        leptos::logging::error!("error: {self}");

        if let ErrorInner::Other { message } = &mut self.error {
            *message = "something went wrong".to_owned();
        }

        let status = match &self.error {
            ErrorInner::ResourceNotFound { .. } => StatusCode::NOT_FOUND,
            ErrorInner::DataConstraint { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorInner::InvalidApiKey
            | ErrorInner::ExpiredApiKey { .. }
            | ErrorInner::NoAuthFound { .. }
            | ErrorInner::InvalidAuthToken
            | ErrorInner::PermissionDenied => StatusCode::UNAUTHORIZED,
            ErrorInner::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl From<TokioPgError> for Error {
    fn from(err: TokioPgError) -> Self {
        let Some(db_error) = err.as_db_error() else {
            return Self::other(err.to_string());
        };

        // TODO: complete this with the relevant SQL states
        let error = match *db_error.code() {
            SqlState::INSUFFICIENT_PRIVILEGE => ErrorInner::PermissionDenied,
            SqlState::CHECK_VIOLATION => ErrorInner::DataConstraint {
                resource: db_error.table().map(str::to_owned),
                field: db_error.column().map(str::to_owned),
                message: db_error.message().to_owned(),
                detail: db_error.detail().map(str::to_owned),
            },
            _ => ErrorInner::Other {
                message: db_error.to_string(),
            },
        };

        Self { error }
    }
}

impl From<DeadpoolPgError> for Error {
    fn from(err: DeadpoolPgError) -> Self {
        Self {
            error: ErrorInner::Other {
                message: err.to_string(),
            },
        }
    }
}
