use uuid::Uuid;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "DatabaseError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
pub enum Error {
    #[error("failed to find {resource} with ID {resource_id}")]
    ResourceNotFound { resource: String, resource_id: Uuid },
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
    #[error("{message}")]
    Other { message: String },
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        use diesel::result::Error::{DatabaseError, NotFound};

        match err {
            DatabaseError(kind, info) => Self::from((kind, info)),
            // Return default values because the error-handling up the chain will populate the
            // correct values
            NotFound => Self::ResourceNotFound {
                resource: String::new(),
                resource_id: Uuid::nil(),
            },
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

impl From<deadpool_diesel::PoolError> for Error {
    fn from(value: deadpool_diesel::PoolError) -> Self {
        Self::Other {
            message: value.to_string(),
        }
    }
}

impl From<deadpool_diesel::InteractError> for Error {
    fn from(value: deadpool_diesel::InteractError) -> Self {
        Self::Other {
            message: value.to_string(),
        }
    }
}
