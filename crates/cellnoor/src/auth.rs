use axum::{extract::FromRequestParts, http::HeaderValue};
use uuid::Uuid;

use crate::{auth::api_key::fetch_api_key_record, db, error::Error, state::AppState};

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
        let Some(api_key) = parts.headers.get("x-api-key").map(HeaderValue::as_bytes) else {
            return Err(Error::no_auth_found(
                "API key must be located in header 'x-api-key'",
            ));
        };

        let api_key_record =
            fetch_api_key_record(api_key, state.db_client(db::User::App).await?).await?;

        api_key_record.to_user()
    }
}
