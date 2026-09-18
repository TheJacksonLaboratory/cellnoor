use aide::{
    OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use deadpool_postgres::{
    PoolError as DeadpoolPgError,
    tokio_postgres::{Error as TokioPgError, error::SqlState},
};

use crate::error::error_response;

#[derive(Debug, Clone, thiserror::Error, serde::Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DbError {
    #[error("resource not found")]
    ResourceNotFound,
    #[error("invalid {referencing_field} for {referencing_resource}")]
    InvalidReference {
        referencing_resource: String,
        referencing_field: String,
    },
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
        #[serde(skip)]
        message: String,
        #[serde(skip)]
        sql_state: Option<SqlState>,
    },
}

impl DbError {
    pub(crate) fn status(&self) -> StatusCode {
        match self {
            Self::ResourceNotFound => StatusCode::NOT_FOUND,
            Self::DataConstraint { .. } | Self::InvalidReference { .. } => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::PermissionDenied { .. } => StatusCode::UNAUTHORIZED,
            Self::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for DbError {
    fn into_response(self) -> Response {
        error_response(self.status(), self)
    }
}

impl OperationOutput for DbError {
    type Inner = Self;

    fn operation_response(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Option<OpenApiResponse> {
        Json::<Self>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        let Some(response) = Self::operation_response(ctx, operation) else {
            return Vec::new();
        };

        vec![(None, response)]
    }
}

impl From<TokioPgError> for DbError {
    fn from(err: TokioPgError) -> Self {
        let Some(db_error) = err.as_db_error() else {
            return Self::Other {
                message: err.to_string(),
                sql_state: None,
            };
        };

        // TODO: complete this with the relevant SQL states
        match db_error.code().clone() {
            SqlState::INSUFFICIENT_PRIVILEGE => Self::PermissionDenied {
                message: db_error.message().replace("table", "resource"),
            },
            SqlState::FOREIGN_KEY_VIOLATION => {
                let referencing_resource = db_error.table().unwrap_or_default();
                let constraint = db_error.constraint().unwrap_or_default();

                Self::InvalidReference {
                    referencing_field: referencing_field(referencing_resource, constraint)
                        .to_owned(),
                    referencing_resource: referencing_resource.to_owned(),
                }
            }
            SqlState::CHECK_VIOLATION | SqlState::UNIQUE_VIOLATION => Self::DataConstraint {
                resource: db_error.table().map(str::to_owned),
                field: db_error.column().map(str::to_owned),
                message: db_error.message().to_owned(),
                detail: db_error.detail().map(str::to_owned),
            },
            sql_state => Self::Other {
                message: db_error.to_string(),
                sql_state: Some(sql_state),
            },
        }
    }
}

impl From<DeadpoolPgError> for DbError {
    fn from(err: DeadpoolPgError) -> Self {
        Self::Other {
            message: err.to_string(),
            sql_state: None,
        }
    }
}

fn referencing_field<'a>(table: &str, constraint: &'a str) -> &'a str {
    constraint
        .strip_prefix(&format!("{table}_"))
        .and_then(|field| field.strip_suffix("_fkey"))
        .unwrap_or(constraint)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::{
        auth::AuthError,
        db::DbError,
        handlers::{file_auth::FileAuthError, people::PersonError},
    };

    fn body<E: serde::Serialize>(error: E) -> serde_json::Value {
        serde_json::to_value(error).unwrap()
    }

    #[test]
    fn composed_errors_serialize_flat() {
        let not_found = json!({ "type": "resource_not_found" });

        assert_eq!(body(DbError::ResourceNotFound), not_found);
        assert_eq!(body(AuthError::Db(DbError::ResourceNotFound)), not_found);
        assert_eq!(
            body(FileAuthError::Db(DbError::ResourceNotFound)),
            not_found
        );

        assert_eq!(
            body(DbError::PermissionDenied {
                message: "nope".to_owned()
            }),
            json!({ "type": "permission_denied", "message": "nope" })
        );
        assert_eq!(
            body(PersonError::InvalidEmail {
                email: "a@b".to_owned()
            }),
            json!({ "type": "invalid_email", "email": "a@b" })
        );
    }

    #[test]
    fn internal_errors_tell_the_client_nothing() {
        assert_eq!(
            body(DbError::Other {
                message: "relation \"person\" does not exist".to_owned(),
                sql_state: None,
            }),
            json!({ "type": "other" })
        );
    }
}
