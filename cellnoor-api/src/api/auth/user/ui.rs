use serde::Deserialize;

use super::{AuthProjects, AuthUser, FromEncodedJwt, common::*};

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    #[allow(dead_code)]
    #[serde(flatten)]
    standard_claims: StandardClaims,
    user: PrivateClaims,
    projects: AuthProjects,
}

impl User {
    pub fn into_authenticated_user(self) -> AuthUser {
        let Self { user, projects, .. } = self;

        AuthUser { user, projects }
    }
}

impl FromEncodedJwt for User {}
