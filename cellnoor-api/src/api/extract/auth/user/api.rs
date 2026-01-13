use std::sync::LazyLock;

use axum::{RequestPartsExt, extract::FromRequestParts};
use axum_extra::TypedHeader;
use cellnoor_schema::api_keys;
use diesel::{PgConnection, prelude::*};
use headers::authorization::Bearer;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use uuid::Uuid;

use super::UserClaims;
use crate::{
    api::{self, extract::auth},
    state::AppState,
};

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct AuthenticatedUser(Uuid);

impl AuthenticatedUser {
    pub fn id(&self) -> Uuid {
        self.0
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = api::ErrorResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState,
    ) -> Result<Self, api::ErrorResponse> {
        let decoding_key = match app_state {
            AppState::Development(state) => {
                return Ok(Self(state.user_id()));
            }
            AppState::Production(state) => state.jwt_decoding_key(),
        };

        let TypedHeader(headers::Authorization::<Bearer>(token)) = parts
            .extract()
            .await
            .map_err(|_| auth::Error::no_auth_token())?;

        let claims = UserClaims::from_jwt(token.token().as_bytes(), decoding_key)?;

        Ok(Self(claims.sub))
    }
}
