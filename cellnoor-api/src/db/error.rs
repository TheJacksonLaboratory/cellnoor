use aide::OperationIo;
use axum::{Json, http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize, JsonSchema, OperationIo)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "DatabaseError"))]
#[serde(rename_all = "snake_case", tag = "type")]
#[schemars(rename = "DatabaseError")]
pub enum Error {
    #[error("{message}")]
    Data { message: String },
    #[error("duplicate {resource} with fields {fields:?} and values {values:?}")]
    DuplicateResource {
        resource: String,
        fields: Vec<String>,
        values: Vec<String>,
    },
    #[error("invalid reference from {resource} to {referenced_resource} with value: {}", value.clone().unwrap_or_default())]
    InvalidReference {
        resource: String,
        referenced_resource: String,
        value: Option<String>,
    },
    #[error("failed to find resource")]
    ResourceNotFound,
    #[error("{message}")]
    Other { message: String },
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        use diesel::result::Error::{DatabaseError, NotFound};

        match err {
            DatabaseError(kind, info) => (kind, info).into(),
            NotFound => Self::ResourceNotFound,
            err => Self::Other {
                message: err.to_string(),
            },
        }
    }
}

impl
    From<(
        diesel::result::DatabaseErrorKind,
        Box<dyn diesel::result::DatabaseErrorInformation + Send + Sync>,
    )> for Error
{
    fn from(
        (kind, info): (
            diesel::result::DatabaseErrorKind,
            Box<dyn diesel::result::DatabaseErrorInformation + Send + Sync>,
        ),
    ) -> Self {
        use diesel::result::DatabaseErrorKind::{
            CheckViolation, ForeignKeyViolation, UniqueViolation,
        };
        use regex::Regex;

        let entity = info.table_name().unwrap_or_default();

        let detail_regex = Regex::new(r"Key \((.+)\)=\((.+)\).+").unwrap(); // This isn't perfect
        let details = info.details().unwrap_or_default();
        let field_value: Vec<String> = detail_regex
            .captures(details)
            .and_then(|cap| {
                cap.iter()
                    .take(3)
                    .map(|m| m.map(|s| s.as_str().to_owned()))
                    .collect()
            })
            .unwrap_or_default();

        let into_split_vecs = |v: &[String], i: usize| {
            v.get(i)
                .cloned()
                .unwrap_or_default()
                .split(", ")
                .map(str::to_string)
                .collect()
        };
        let fields = into_split_vecs(&field_value, 1);
        let values = into_split_vecs(&field_value, 2);

        match kind {
            CheckViolation => Self::Data {
                message: details.to_owned(),
            },
            UniqueViolation => Self::DuplicateResource {
                resource: entity.to_owned(),
                fields,
                values,
            },

            ForeignKeyViolation => {
                let referenced_entity = details
                    .split_whitespace()
                    .last()
                    .unwrap_or_default()
                    .replace('"', "");
                let referenced_entity = referenced_entity.strip_suffix(".").unwrap_or_default();

                Self::InvalidReference {
                    resource: entity.to_owned(),
                    referenced_resource: referenced_entity.to_owned(),
                    value: values.first().cloned(),
                }
            }
            _ => Self::Other {
                message: diesel::result::Error::DatabaseError(kind, info).to_string(),
            },
        }
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for Error {
    fn from(err: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        Self::Other {
            message: err.to_string(),
        }
    }
}

impl Error {
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Data { .. } | Self::InvalidReference { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::DuplicateResource { .. } => StatusCode::CONFLICT,
            Self::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ResourceNotFound => StatusCode::NOT_FOUND,
        }
        .as_u16()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(status = self.status_code(), error = %self);

        let status_code = self.status_code();
        let err = match self {
            Self::Other { message } => Self::Other {
                message: "something went wrong".to_owned(),
            },
            _ => self,
        };

        (StatusCode::from_u16(status_code).unwrap(), Json(err)).into_response()
    }
}
