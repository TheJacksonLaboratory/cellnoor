use aide::OperationIo;
pub use api_key::hash_api_key;
use axum::{RequestPartsExt, extract::FromRequestParts, http::HeaderValue};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{
    auth::{api_key::authenticate_with_api_key, jwt::authenticate_with_jwt},
    error::{Error, ErrorInner},
    state::AppState,
};

mod api_key;
mod jwt;

/// The principal (a person or a service) on whose behalf a request runs.
///
/// Outside of tests, the only way to construct an `AuthUser` is through its
/// `axum::extract::FromRequestParts` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, OperationIo)]
pub struct AuthUser {
    id: Uuid,
    is_staff: bool,
}

impl AuthUser {
    /// The admin user has the nil UUID (see
    /// /db/migrations/0025_insert-admin.up.sql)
    pub(crate) fn admin() -> Self {
        Self {
            id: Uuid::nil(),
            is_staff: true,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn is_staff(&self) -> bool {
        self.is_staff
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Without a JWT secret, authentication is disabled and every request
        // runs as the admin
        let Some((secret, validation)) = state.jwt_decoding_info() else {
            return Ok(Self::admin());
        };

        let cookies = parts.extract::<CookieJar>().await.unwrap();
        let encoded_jwt = read_chunked_jwt(&cookies);
        if !encoded_jwt.is_empty()
            && let Ok(user) = authenticate_with_jwt(encoded_jwt.as_bytes(), secret, validation)
        {
            return Ok(user);
        }

        let Some(api_key) = parts.headers.get("x-api-key").map(HeaderValue::as_bytes) else {
            return Err(ErrorInner::NoAuthFound {
                message: "failed to authenticate with JWT at cookie 'cellnoor-auth.session_data' \
                          and API key at header 'x-api-key'",
            }
            .into());
        };

        Ok(authenticate_with_api_key(state, api_key).await?)
    }
}

fn read_chunked_jwt(cookies: &CookieJar) -> String {
    // The JWT is split across cookies whose names share a prefix (set in
    // /packages/cellnoor-auth/src/auth.ts), and sorting by name puts the chunks
    // back in order. We don't expect both the secure and the insecure prefix to
    // be set at once
    let mut chunks: Vec<_> = cookies
        .iter()
        .filter(|cookie| {
            cookie
                .name()
                .starts_with("__Secure-cellnoor-auth.session_data")
                || cookie.name().starts_with("cellnoor-auth.session_data")
        })
        .map(|cookie| cookie.name_value())
        .collect();

    chunks.sort();

    chunks.into_iter().map(|(_, value)| value).collect()
}

#[cfg(test)]
impl AuthUser {
    pub fn new_as_user(id: Uuid) -> Self {
        Self {
            id,
            is_staff: false,
        }
    }
}
