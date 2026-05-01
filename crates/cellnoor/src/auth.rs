use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    RequestPartsExt,
    extract::{FromRequest, FromRequestParts},
    http::HeaderValue,
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use postgres_types::FromSql;
use uuid::Uuid;

use crate::{
    auth::api_key::{fetch_user_id_by_api_key, verify_api_key},
    db,
    error::{self, Error},
    state::{AppState, ProdState},
};

mod api_key;

/// An alias to [db::User] for convenience.
pub type AuthUser = db::User;

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AppState::Prod(state) = state else {
            // The nil UUID corresponds to the admin user
            return Ok(Self::Person(Uuid::nil()));
        };

        // Eventually, we'll also decode the JWT that better-auth sets, so we will have
        // both UI auth and API auth mechanisms consolidated here. For now, we don't
        // need that
        let Some(Ok(api_key)) = parts.headers.get("x-api-key").map(HeaderValue::to_str) else {
            return Err(Error::no_auth_found(
                "API key must be located in header 'x-api-key'",
            ));
        };

        let hashed_key =
            fetch_user_id_by_api_key(api_key, state.db_client(db::User::App).await?).await?;

        verify_api_key(state.api_key_verifier(), api_key, &hashed_key)?;

        Ok(hashed_key.to_user())
    }
}
