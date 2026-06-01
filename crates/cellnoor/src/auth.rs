use aide::OperationIo;
pub use api_key::hash_api_key;
use axum::{extract::FromRequestParts, http::HeaderValue};
use deadpool_postgres::PoolError;
use uuid::Uuid;

use crate::{
    auth::api_key::fetch_api_key_record,
    db,
    error::{Error, ErrorInner},
    state::{AppState, DevState},
};

mod api_key;

/// An alias to [db::User] for convenience.
// pub type AuthUser = db::User;

/// A database user.
///
/// This could represent a person using the UI, a person using the RESTful API,
/// a service account using the RESTful API, or the app itself switching into
/// one of the aforementioned users.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DbUser {
    App,
    Person(Uuid),
    Service(Uuid),
}

/// An authenticated user.
///
/// The only way to construct an `AuthUser` is through its
/// `axum::extract::FromRequestParts` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, OperationIo)]
pub struct AuthUser(DbUser);

impl std::fmt::Display for AuthUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            DbUser::App => "app".fmt(f),
            DbUser::Service(id) | DbUser::Person(id) => id.fmt(f),
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AppState::Prod(state) = state else {
            // The nil UUID corresponds to the admin user
            return Ok(Self(DbUser::Person(Uuid::nil())));
        };

        // Eventually, we'll also decode the JWT that better-auth sets, so we will have
        // both UI auth and API auth mechanisms consolidated here. For now, we don't
        // need that
        let Some(api_key) = parts.headers.get("x-api-key").map(HeaderValue::as_bytes) else {
            return Err(ErrorInner::NoAuthFound {
                message: "API key must be located in header 'x-api-key'",
            }
            .into());
        };

        let api_key_record =
            fetch_api_key_record(api_key, state.db_client(Self(DbUser::App)).await?).await?;

        api_key_record.to_user().map_err(Error::from)
    }
}

#[cfg(test)]
impl AuthUser {
    pub fn new_as_app() -> Self {
        Self(DbUser::App)
    }

    pub fn new_as_admin() -> Self {
        Self(DbUser::Person(Uuid::nil()))
    }

    pub fn new_as_user(id: Uuid) -> Self {
        Self(DbUser::Person(id))
    }
}

// This impl has to go here
impl DevState {
    pub async fn db_client(&self) -> Result<db::Client, PoolError> {
        self.db_pool()
            .get(AuthUser(DbUser::Person(Uuid::nil())))
            .await
    }
}
