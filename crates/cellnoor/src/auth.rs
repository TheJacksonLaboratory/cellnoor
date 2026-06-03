use aide::OperationIo;
pub use api_key::hash_api_key;
use axum::{RequestPartsExt, extract::FromRequestParts, http::HeaderValue};
use axum_extra::extract::{CookieJar, cookie::Cookie};

use deadpool_postgres::PoolError;
use uuid::Uuid;

use crate::{
    auth::{api_key::authenticate_with_api_key, jwt::authenticate_with_jwt},
    db,
    error::{Error, ErrorInner},
    state::{AppState, DevState},
};

mod api_key;
mod jwt;

/// A database user.
///
/// This could represent a person using the UI, a person using the RESTful API,
/// a service account using the RESTful API, or the app itself switching into
/// one of the aforementioned users.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DbUser {
    App,
    ApiKey(Uuid),
    Jwt { user_id: Uuid, is_staff: bool },
}

/// An authenticated user.
///
/// The only way to construct an `AuthUser` is through its
/// `axum::extract::FromRequestParts` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, OperationIo)]
pub struct AuthUser(DbUser);

impl AuthUser {
    pub fn id(&self) -> Option<Uuid> {
        match self.0 {
            DbUser::App => None,
            DbUser::Jwt { user_id: id, .. } | DbUser::ApiKey(id) => Some(id),
        }
    }

    pub fn is_staff(&self) -> Option<bool> {
        if let Self(DbUser::Jwt {
            user_id: _,
            is_staff,
        }) = self
        {
            return Some(*is_staff);
        }

        None
    }
}

impl std::fmt::Display for AuthUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id()
            .map(|id| std::fmt::Display::fmt(&id, f))
            .unwrap_or_else(|| "app".fmt(f))
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
            return Ok(Self(DbUser::ApiKey(Uuid::nil())));
        };

        let cookies = parts.extract::<CookieJar>().await.unwrap();
        // We know the name of the cookie because it's set in
        // packages/cellnoor-auth/src/auth.ts
        if let Some(encoded_jwt) = cookies
            .get("__Secure-cellnoor-auth.session_data")
            .or(cookies.get("cellnoor-auth.session_data"))
            .map(Cookie::value)
        {
            let (secret, validation) = state.jwt_decoding_info();
            match authenticate_with_jwt(encoded_jwt.as_bytes(), secret, validation) {
                Ok(user) => return Ok(user),
                // Do nothing with the error for now
                Err(_) => (),
            }
        }

        // Eventually, we'll also decode the JWT that better-auth sets, so we will have
        // both UI auth and API auth mechanisms consolidated here. For now, we don't
        // need that
        let Some(api_key) = parts.headers.get("x-api-key").map(HeaderValue::as_bytes) else {
            return Err(ErrorInner::NoAuthFound {
                message: "failed to authenticate with JWT at cookie 'cellnoor-auth.session_data' and API key at header 'x-api-key'",
            }
            .into());
        };

        Ok(authenticate_with_api_key(state, api_key).await?)
    }
}

#[cfg(test)]
impl AuthUser {
    pub fn new_as_app() -> Self {
        Self(DbUser::App)
    }

    pub fn new_as_admin() -> Self {
        Self(DbUser::Jwt {
            user_id: Uuid::nil(),
            is_staff: true,
        })
    }

    pub fn new_as_user(user_id: Uuid) -> Self {
        Self(DbUser::Jwt {
            user_id,
            is_staff: false,
        })
    }
}

// This impl has to go here
impl DevState {
    pub async fn db_client(&self) -> Result<db::Client, PoolError> {
        self.db_pool()
            .get(AuthUser(DbUser::ApiKey(Uuid::nil())))
            .await
    }
}
