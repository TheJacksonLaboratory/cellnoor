use aide::OperationIo;
use axum::{Json, http::StatusCode, response::IntoResponse};
use deadpool_postgres::{
    PoolError as DeadpoolPgError,
    tokio_postgres::{Error as TokioPgError, error::SqlState},
};

#[derive(
    Debug,
    Clone,
    thiserror::Error,
    serde::Serialize,
    schemars::JsonSchema,
    OperationIo,
    PartialEq,
    Eq,
)]
#[error(transparent)]
pub struct Error {
    pub error: ErrorInner,
}

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ErrorInner {
    #[error("resource not found")]
    ResourceNotFound,
    #[error("invalid API key")]
    InvalidApiKey,
    #[error("invalid {referencing_field} for {referencing_resource}")]
    InvalidReference {
        referencing_resource: String,
        referencing_field: String,
    },
    #[error("API key expired at {expired_at}")]
    ExpiredApiKey { expired_at: jiff::Timestamp },
    #[error("invalid auth token: {message}")]
    InvalidAuthToken { message: String },
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
    PermissionDenied { message: String },
    #[error("{message} (SQL State - {})", sql_state.as_ref().map(|s| s.code()).unwrap_or_default())]
    Other {
        message: String,
        #[serde(skip)]
        sql_state: Option<SqlState>,
    },
}

impl IntoResponse for Error {
    fn into_response(mut self) -> axum::response::Response {
        // TODO: add logging
        if let ErrorInner::Other {
            message,
            sql_state: _,
        } = &mut self.error
        {
            *message = "something went wrong".to_owned();
        }

        let status = match &self.error {
            ErrorInner::ResourceNotFound => StatusCode::NOT_FOUND,
            ErrorInner::DataConstraint { .. } | ErrorInner::InvalidReference { .. } => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            ErrorInner::InvalidApiKey
            | ErrorInner::ExpiredApiKey { .. }
            | ErrorInner::NoAuthFound { .. }
            | ErrorInner::InvalidAuthToken { .. }
            | ErrorInner::PermissionDenied { .. } => StatusCode::UNAUTHORIZED,
            ErrorInner::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl From<TokioPgError> for ErrorInner {
    fn from(err: TokioPgError) -> Self {
        let Some(db_error) = err.as_db_error() else {
            return ErrorInner::Other {
                message: err.to_string(),
                sql_state: None,
            };
        };

        // TODO: complete this with the relevant SQL states
        match db_error.code().clone() {
            SqlState::INSUFFICIENT_PRIVILEGE => ErrorInner::PermissionDenied {
                message: db_error.message().replace("table", "resource"),
            },
            SqlState::FOREIGN_KEY_VIOLATION => {
                let referencing_resource = db_error.table().map(str::to_owned).unwrap();
                let referencing_field_prefix = format!("{referencing_resource}_");

                ErrorInner::InvalidReference {
                    referencing_resource: db_error.table().map(str::to_owned).unwrap(),
                    // This looks insane but it's just basically transforming something like
                    // 'person_institution_id_fkey' to 'institution_id'
                    referencing_field: db_error
                        .constraint()
                        .unwrap()
                        .strip_prefix(&referencing_field_prefix)
                        .unwrap()
                        .strip_suffix("_fkey")
                        .unwrap()
                        .to_owned(),
                }
            }
            SqlState::CHECK_VIOLATION | SqlState::UNIQUE_VIOLATION => ErrorInner::DataConstraint {
                resource: db_error.table().map(str::to_owned),
                field: db_error.column().map(str::to_owned),
                message: db_error.message().to_owned(),
                detail: db_error.detail().map(str::to_owned),
            },
            sql_state => ErrorInner::Other {
                message: db_error.to_string(),
                sql_state: Some(sql_state),
            },
        }
    }
}

impl From<ErrorInner> for Error {
    fn from(error: ErrorInner) -> Self {
        Self { error }
    }
}

impl From<DeadpoolPgError> for ErrorInner {
    fn from(err: DeadpoolPgError) -> Self {
        Self::Other {
            message: err.to_string(),
            sql_state: None,
        }
    }
}

impl From<TokioPgError> for Error {
    fn from(err: TokioPgError) -> Self {
        Self { error: err.into() }
    }
}

impl From<DeadpoolPgError> for Error {
    fn from(err: DeadpoolPgError) -> Self {
        Self { error: err.into() }
    }
}
