use axum::{RequestPartsExt, extract::FromRequestParts};
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, cookie::Cookie},
};
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{TokenData, Validation};
use serde::de::DeserializeOwned;
use tokio::sync::RwLockReadGuard;

use crate::{
    api::{ErrorResponse, extract::auth},
    state::{AppState, JwtDecodingKey},
};

mod api;
mod ui;

// We don't implement any function to get the user ID so as to statically ensure
// that the authorization function checks whether this is an API user or a UI
// user
#[derive(Clone, Debug, serde::Deserialize)]
pub enum AuthenticatedUser {
    Api(api::User),
    Ui(ui::User),
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState,
    ) -> Result<Self, ErrorResponse> {
        let (decoding_key, validation) = match app_state {
            AppState::Development(_) => return Ok(Self::Ui(ui::User::admin())),
            AppState::Production(state) => {
                (state.jwt_decoding_key().await?, state.jwt_validation())
            }
        };

        let auth_header = extract_auth_header(parts).await;
        let cookies = extract_cookies(parts).await;
        let encoded_jwt = extract_jwt_from_header_or_cookies(auth_header.as_ref(), &cookies)?;

        let user = match encoded_jwt {
            EncodedJwt::FromAuthorizationHeader(t) => {
                api::User::from_encoded_jwt(t, &decoding_key, validation).map(Self::Api)?
            }
            EncodedJwt::FromCookie(t) => {
                ui::User::from_encoded_jwt(t, &decoding_key, validation).map(Self::Ui)?
            }
        };

        tracing::info!("authenticated: {user:?}");

        Ok(user)
    }
}

async fn extract_auth_header(
    request_parts: &mut axum::http::request::Parts,
) -> Option<TypedHeader<Authorization<Bearer>>> {
    request_parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .ok()
}

async fn extract_cookies(request_parts: &mut axum::http::request::Parts) -> CookieJar {
    request_parts
        .extract()
        .await
        .expect("should be able to extract cookies")
}

enum EncodedJwt<'a> {
    FromAuthorizationHeader(&'a [u8]),
    FromCookie(&'a [u8]),
}

fn extract_jwt_from_header_or_cookies<'a>(
    auth_header: Option<&'a TypedHeader<Authorization<Bearer>>>,
    cookies: &'a CookieJar,
) -> Result<EncodedJwt<'a>, auth::Error> {
    match (auth_header, cookies) {
        (Some(TypedHeader(Authorization(token))), _) => Ok(EncodedJwt::FromAuthorizationHeader(
            token.token().as_bytes(),
        )),
        (None, cookies) => cookies
            .get("cellnoor-ui.api_token")
            .map(Cookie::value)
            .map(str::as_bytes)
            .map(EncodedJwt::FromCookie)
            .ok_or(auth::Error::no_auth_token()),
    }
}

trait FromEncodedJwt: DeserializeOwned {
    fn from_encoded_jwt(
        encoded_jwt: &[u8],
        decoding_key: &RwLockReadGuard<Option<JwtDecodingKey>>,
        validation: &Validation,
    ) -> Result<Self, super::Error> {
        let TokenData { header: _, claims } = jsonwebtoken::decode(
            encoded_jwt,
            decoding_key
                .as_ref()
                .map(JwtDecodingKey::public_key)
                .unwrap(),
            validation,
        )?;

        Ok(claims)
    }
}
