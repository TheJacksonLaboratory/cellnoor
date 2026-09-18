use std::str::FromStr;

use aide::{
    OperationInput, OperationOutput,
    generate::GenContext,
    openapi::{Operation, Response as OpenApiResponse, StatusCode as OpenApiStatusCode},
};
pub use api_key::hash_api_key;
use axum::{Json, RequestPartsExt, extract::FromRequestParts, http::HeaderValue};
use axum_extra::extract::CookieJar;
pub use error::AuthError;
use uuid::Uuid;

use crate::{
    auth::{api_key::authenticate_with_api_key, jwt::authenticate_with_jwt},
    state::AppState,
};

mod api_key;
mod error;
mod jwt;

/// The principal (a person or a service) on whose behalf a request runs.
///
/// Outside of tests, the only way to construct an `AuthUser` is through its
/// `axum::extract::FromRequestParts` implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl OperationInput for AuthUser {
    fn inferred_early_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<OpenApiStatusCode>, OpenApiResponse)> {
        let Some(response) = Json::<AuthError>::operation_response(ctx, operation) else {
            return Vec::new();
        };

        vec![(Some(OpenApiStatusCode::Code(401)), response)]
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Without a JWT secret, authentication is disabled and every request
        // runs as the admin
        let Some((secret, validation)) = state.jwt_decoding_info else {
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
            return Err(AuthError::NoAuthFound {
                message: "failed to authenticate with JWT at cookie 'cellnoor-auth.session_data' \
                          and API key at header 'x-api-key'",
            });
        };

        authenticate_with_api_key(state, api_key).await
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

    let number_part = |(_, n): (&str, &str)| u8::from_str(n).ok();

    chunks.sort_by_key(|(cookiename, _)| cookiename.rsplit_once('.').map(number_part));

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

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;
    use axum_extra::extract::CookieJar;
    use pretty_assertions::assert_str_eq;

    use crate::auth::read_chunked_jwt;

    fn cookie_jar(header: &str) -> CookieJar {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", header.parse().unwrap());

        CookieJar::from_headers(&headers)
    }

    #[test]
    fn chunks_are_joined_in_name_order() {
        let jar = cookie_jar(
            "cellnoor-auth.session_data.1=second; unrelated=ignored; \
             cellnoor-auth.session_data.0=first; cellnoor-auth.session_data.10=tenth",
        );

        assert_str_eq!(read_chunked_jwt(&jar), "firstsecondtenth");
    }

    #[test]
    fn secure_chunks_are_joined_in_name_order() {
        let jar = cookie_jar(
            "__Secure-cellnoor-auth.session_data.1=second; \
             __Secure-cellnoor-auth.session_data.0=first",
        );

        assert_str_eq!(read_chunked_jwt(&jar), "firstsecond");
    }

    #[test]
    fn no_matching_cookie_reads_as_empty() {
        assert_str_eq!(read_chunked_jwt(&cookie_jar("unrelated=ignored")), "");
    }
}
