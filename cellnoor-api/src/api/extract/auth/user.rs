use axum::{RequestPartsExt, extract::FromRequestParts};
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, cookie::Cookie},
};
use diesel_async::AsyncPgConnection;
use headers::{Authorization as AuthorizationHeader, authorization::Bearer};
use jsonwebtoken::{TokenData, Validation};
use serde::de::DeserializeOwned;
use tokio::sync::RwLockReadGuard;

use crate::{
    api::extract::auth,
    db::{self, DbConnection},
    state::{AppState, JwtDecodingKey},
};
pub use common::Authorization;

mod api;
mod common;
mod ui;

// We don't implement any function to get the user ID so as to statically ensure
// that the authorization function checks whether this is an API user or a UI
// user
#[derive(Clone, Debug)]
pub enum AuthenticatedUser {
    Api(api::User),
    Ui(ui::User),
}

impl AuthenticatedUser {
    pub async fn authorization(
        self,
        db_conn: &AsyncPgConnection,
    ) -> Result<Authorization, db::Error> {
        match self {
            Self::Api(u) => u.authorization(db_conn).await,
            Self::Ui(u) => Ok(u.into_authorization()),
        }
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = crate::api::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState,
    ) -> Result<Self, crate::api::Error> {
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

        Ok(user)
    }
}

async fn extract_auth_header(
    request_parts: &mut axum::http::request::Parts,
) -> Option<TypedHeader<AuthorizationHeader<Bearer>>> {
    request_parts
        .extract::<TypedHeader<AuthorizationHeader<Bearer>>>()
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
    auth_header: Option<&'a TypedHeader<AuthorizationHeader<Bearer>>>,
    cookies: &'a CookieJar,
) -> Result<EncodedJwt<'a>, auth::Error> {
    match (auth_header, cookies) {
        (Some(TypedHeader(AuthorizationHeader(token))), _) => Ok(
            EncodedJwt::FromAuthorizationHeader(token.token().as_bytes()),
        ),
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
