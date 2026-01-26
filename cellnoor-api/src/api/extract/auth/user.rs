use std::collections::HashSet;

use axum::{RequestPartsExt, extract::FromRequestParts};
use axum_extra::{
    TypedHeader,
    extract::{CookieJar, cookie::Cookie},
};
use diesel_async::AsyncPgConnection;
use headers::{Authorization as AuthorizationHeader, authorization::Bearer};
use jsonwebtoken::{TokenData, Validation};
use serde::{Deserialize, de::DeserializeOwned};
use tokio::sync::RwLockReadGuard;
use uuid::Uuid;

use crate::{
    api::{auth::user::common::PrivateClaims, extract::auth},
    db::{self, DbConnection},
    state::{AppState, JwtDecodingKey},
};

mod api;
mod common;
mod ui;

#[derive(Clone, Debug, Deserialize)]
pub struct AuthenticatedUser {
    user: PrivateClaims,
    projects: HashSet<Uuid>,
}

impl AuthenticatedUser {
    pub fn new_admin() -> Self {
        Self {
            user: PrivateClaims {
                user_id: Uuid::nil(),
                is_admin: true,
                is_biology_staff: true,
                is_computational_staff: true,
            },
            projects: HashSet::new(),
        }
    }

    fn data(&self) -> &PrivateClaims {
        &self.user
    }

    pub fn is_admin(&self) -> bool {
        self.data().is_admin
    }

    pub fn is_biology_staff(&self) -> bool {
        self.data().is_biology_staff
    }

    pub fn is_computational_staff(&self) -> bool {
        self.data().is_computational_staff
    }

    pub fn is_staff(&self) -> bool {
        self.is_admin() || self.is_biology_staff() || self.is_computational_staff()
    }

    pub fn authorize_admin_only(&self) -> Result<(), auth::Error> {
        if !self.is_admin() {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(())
    }

    pub fn authorized_projects(
        self,
        requested_projects: Option<HashSet<Uuid>>,
    ) -> Option<HashSet<Uuid>> {
        if self.is_staff() {
            return requested_projects;
        }

        let authorized_projects = self.projects;

        let Some(requested_projects) = requested_projects else {
            return Some(authorized_projects);
        };

        Some(
            requested_projects
                .into_iter()
                .filter(|p| authorized_projects.contains(p))
                .collect(),
        )
    }

    pub fn authorize_project_access(self, requested_project: &Uuid) -> Result<(), auth::Error> {
        let authorized_projects = self
            .authorized_projects(Some(HashSet::from([*requested_project])))
            .expect("we pased in a project, so we should get one out");

        if !authorized_projects.contains(requested_project) {
            return Err(auth::Error::PermissionDenied);
        }

        Ok(())
    }
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = crate::api::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        app_state: &AppState,
    ) -> Result<Self, crate::api::Error> {
        let (decoding_key, validation) = match app_state {
            AppState::Development(_) => return Ok(Self::new_admin()),
            AppState::Production(state) => {
                (state.jwt_decoding_key().await?, state.jwt_validation())
            }
        };

        let auth_header = extract_auth_header(parts).await;
        let cookies = extract_cookies(parts).await;
        let encoded_jwt = extract_jwt_from_header_or_cookies(auth_header.as_ref(), &cookies)?;

        let user = match encoded_jwt {
            EncodedJwt::FromAuthorizationHeader(t) => {
                let api_user = api::User::from_encoded_jwt(t, &decoding_key, validation)?;
                let db_conn = app_state.db_conn().await?;
                api_user.with_authorized_projects(&db_conn).await?
            }
            EncodedJwt::FromCookie(t) => Self::from_encoded_jwt(t, &decoding_key, validation)?,
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

impl FromEncodedJwt for AuthenticatedUser {}
