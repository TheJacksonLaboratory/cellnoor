use serde::Deserialize;

use super::{AuthProjects, AuthUser, FromEncodedJwt, common::*};

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    #[allow(dead_code)]
    #[serde(flatten)]
    standard_claims: StandardClaims,
    private_claims: PrivateClaims,
    projects: AuthProjects,
}

impl User {
    pub fn into_authenticated_user(self) -> AuthUser {
        let Self {
            private_claims,
            projects,
            ..
        } = self;

        AuthUser {
            user: private_claims,
            projects,
        }
    }
}

impl FromEncodedJwt for User {}
