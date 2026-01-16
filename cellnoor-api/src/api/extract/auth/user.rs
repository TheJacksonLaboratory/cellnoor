use std::{collections::HashMap, sync::LazyLock};

use axum::{RequestPartsExt, extract::FromRequestParts};
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, cookie::Cookie},
};
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, jwk::JwkSet};
use serde::{Deserialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::{
    api::{ErrorResponse, extract::auth},
    state::AppState,
};

mod api;
mod ui;

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct AuthenticatedUser(Uuid);

impl AuthenticatedUser {
    pub fn id(&self) -> Uuid {
        self.0
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState,
    ) -> Result<Self, ErrorResponse> {
        let decoding_key = match app_state {
            AppState::Development(state) => {
                return Ok(Self(state.user_id()));
            }
            AppState::Production(state) => state.jwt_decoding_key(),
        };

        let auth_header = parts.extract::<TypedHeader<Authorization<Bearer>>>().await;
        let cookies = parts.extract::<CookieJar>().await;
        let token_locations = (auth_header, cookies);

        let token = match &token_locations {
            (Ok(TypedHeader(Authorization(token))), _) => token.token().as_bytes(),
            (Err(_), Ok(cookies)) => cookies
                .get("cellnoor-ui.api_token")
                .map(Cookie::value)
                .map(str::as_bytes)
                .unwrap_or_default(),
            (Err(_), Err(_)) => return Err(auth::Error::no_auth_token())?,
        };

        let claims = UserClaims::from_jwt(token, decoding_key)?;

        Ok(Self(claims.sub))
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct UserClaims {
    sub: Uuid,
    exp: usize,
    jti: Uuid,
}

impl UserClaims {
    fn from_jwt(encoded_jwt: &[u8], decoding_key: &DecodingKey) -> Result<Self, super::Error> {
        static VALIDATION: LazyLock<Validation> =
            LazyLock::new(|| Validation::new(Algorithm::HS512));

        let token = jsonwebtoken::decode(encoded_jwt, decoding_key, &VALIDATION)?;

        Ok(token.claims)
    }
}

trait FromJwt: DeserializeOwned {
    fn from_jwt(
        encoded_jwt: &[u8],
        decoding_keys: &HashMap<DecodingKey>,
        validation: &Validation,
    ) -> Result<Self, super::Error> {
    }
}
