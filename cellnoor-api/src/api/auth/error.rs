use crate::db;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(rename = "AuthError"))]
#[serde(rename_all = "snake_case", tag = "type", content = "info")]
#[error(transparent)]
pub enum Error {
    #[error("no auth token found")]
    NoAuthTokenFound {
        message: &'static str,
    },
    #[error("invalid auth token")]
    InvalidAuthToken {
        message: String,
    },
    #[error("permission denied")]
    PermissionDenied,
    Database(#[from] db::Error),
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::InvalidAuthToken {
            message: err.to_string(),
        }
    }
}

impl Error {
    pub fn no_auth_token() -> Self {
        Self::NoAuthTokenFound {
            message: "no authorization token found in cookies nor in 'Authorization: Bearer' \
                      header",
        }
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for Error {
    fn from(err: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        Self::Database(err.into())
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self {
        Self::Database(err.into())
    }
}
