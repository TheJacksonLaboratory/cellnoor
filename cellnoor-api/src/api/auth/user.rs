use std::{collections::HashSet, sync::Arc};

use axum_extra::{
    TypedHeader,
    extract::{CookieJar, cookie::Cookie},
};
use headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{TokenData, Validation};
use serde::{Deserialize, de::DeserializeOwned};
use std::sync::RwLockReadGuard;
use uuid::Uuid;

use crate::{
    api::auth::user::common::PrivateClaims,
    db,
    state::{AppState, JwtDecodingKey},
};

pub(super) mod api;
mod common;
mod ui;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser(Arc<AuthenticatedUserInner>);

#[derive(Clone, Debug, Deserialize)]
struct AuthenticatedUserInner {
    user: PrivateClaims,
    projects: HashSet<Uuid>,
}

impl AuthenticatedUser {
    pub async fn from_request(
        app_state: &AppState,
        auth_header: Option<&TypedHeader<Authorization<Bearer>>>,
        cookies: &CookieJar,
    ) -> Result<Self, super::Error> {
        let (decoding_key, validation) = match &app_state {
            AppState::Development(_) => {
                return Ok(Self::new_admin());
            }
            AppState::Production(state) => (
                state.jwt_decoding_key().await.map_err(|e| {
                    super::Error::Database(db::Error::Other {
                        message: format!("failed to fetch JWK from database: {e}"),
                    })
                })?,
                state.jwt_validation(),
            ),
        };

        let encoded_jwt = extract_jwt_from_header_or_cookies(auth_header, cookies)?;

        let user = match encoded_jwt {
            EncodedJwt::FromAuthorizationHeader(t) => {
                let api_user = api::User::from_encoded_jwt(t, decoding_key, validation)?;
                let db_conn = app_state.db_conn().await?;
                api_user.with_authorized_projects(&db_conn).await?
            }
            EncodedJwt::FromCookie(t) => {
                let ui_user = ui::User::from_encoded_jwt(t, decoding_key, validation)?;
                ui_user.into_authenticated_user()
            }
        };

        Ok(Self(Arc::new(user)))
    }

    pub fn new_admin() -> Self {
        Self(Arc::new(AuthenticatedUserInner {
            user: PrivateClaims {
                user_id: Uuid::nil(),
                is_admin: true,
                is_biology_staff: true,
                is_computational_staff: true,
            },
            projects: HashSet::new(),
        }))
    }

    fn data(&self) -> &PrivateClaims {
        &self.0.user
    }

    pub fn projects(&self) -> &HashSet<Uuid> {
        &self.0.projects
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

    pub fn authorized_projects(&self) -> Option<&HashSet<Uuid>> {
        (!self.is_staff()).then_some(self.projects())
    }
}

enum EncodedJwt<'a> {
    FromAuthorizationHeader(&'a [u8]),
    FromCookie(&'a [u8]),
}

fn extract_jwt_from_header_or_cookies<'a>(
    auth_header: Option<&'a TypedHeader<Authorization<Bearer>>>,
    cookies: &'a CookieJar,
) -> Result<EncodedJwt<'a>, super::Error> {
    match (auth_header, cookies) {
        (Some(TypedHeader(Authorization(token))), _) => Ok(EncodedJwt::FromAuthorizationHeader(
            token.token().as_bytes(),
        )),
        (None, cookies) => cookies
            .get("cellnoor-ui.api_token")
            .map(Cookie::value)
            .map(str::as_bytes)
            .map(EncodedJwt::FromCookie)
            .ok_or(super::Error::no_auth_token()),
    }
}

trait FromEncodedJwt: DeserializeOwned {
    fn from_encoded_jwt(
        encoded_jwt: &[u8],
        decoding_key: RwLockReadGuard<Option<JwtDecodingKey>>,
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

pub trait RemoveUnauthorizedProjects {
    fn remove_unauthorized_projects(&mut self, user: &AuthenticatedUser);
}

impl RemoveUnauthorizedProjects for Option<Vec<Uuid>> {
    fn remove_unauthorized_projects(&mut self, user: &AuthenticatedUser) {
        // Staff can view whatever they want
        if user.is_staff() {
            return;
        }

        let Some(requested_projects) = self.as_mut() else {
            // If there were no requested projects, then it should just be the projects the user is authorized to view. Also this copy is unavoidable
            self.replace(user.projects().iter().copied().collect());
            return;
        };

        for requested_project in requested_projects {
            if !user.projects().contains(requested_project) {
                // We're banking on the fact that there are no projects with nil UUIDs
                *requested_project = Uuid::nil();
            }
        }
    }
}
